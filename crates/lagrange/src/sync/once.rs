use core::{
    cell::Cell,
    convert::Infallible,
    fmt::Debug,
    hint::unreachable_unchecked,
    marker::PhantomData,
    sync::atomic::{AtomicBool, AtomicPtr, Ordering},
};

use crate::thread::{self, Thread};

/// A primitive for one-time initialization.
///
/// This is very similar to the `std::sync` version, except with the addition of support
/// for `try_call_once()`.
pub struct Once {
    // This `queue` field is the core of the implementation. It encodes two
    // pieces of information:
    //
    // * The current state of the cell (`INCOMPLETE`, `RUNNING`, `COMPLETE`)
    // * Linked list of threads waiting for the current cell.
    //
    // State is encoded in two low bits. Only `INCOMPLETE` and `RUNNING` states
    // allow waiters.
    queue: AtomicPtr<Waiter>,
    _marker: PhantomData<*mut Waiter>,
}

unsafe impl Sync for Once {}
unsafe impl Send for Once {}

impl Once {
    pub const fn new() -> Once {
        Once {
            queue: AtomicPtr::new(INCOMPLETE_PTR),
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn try_call_once<F, E>(&self, f: F) -> Result<(), E>
    where
        F: FnOnce() -> Result<(), E>,
    {
        if self.is_completed() {
            Ok(())
        } else {
            self.try_call_once_slow(f)
        }
    }

    #[cold]
    fn try_call_once_slow<F, E>(&self, f: F) -> Result<(), E>
    where
        F: FnOnce() -> Result<(), E>,
    {
        let mut f = Some(f);
        let mut res: Option<Result<(), E>> = None;

        initialize_or_wait(
            &self.queue,
            Some(&mut || {
                let f = unsafe { unwrap_unchecked(f.take()) };
                res.insert(f()).is_ok()
            }),
        );

        res.unwrap()
    }

    #[inline]
    pub fn call_once<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        self.try_call_once(|| {
            f();
            Ok::<_, Infallible>(())
        })
        .unwrap()
    }

    #[inline]
    pub fn is_completed(&self) -> bool {
        // An `Acquire` load is enough because that makes all the initialization
        // operations visible to us, and, this being a fast path, weaker
        // ordering helps with performance. This `Acquire` synchronizes with
        // `SeqCst` operations on the slow path.
        self.queue.load(Ordering::Acquire) == COMPLETE_PTR
    }

    #[cold]
    pub fn wait(&self) {
        initialize_or_wait(&self.queue, None);
    }
}

impl Debug for Once {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Once")
            .field("is_completed", &self.is_completed())
            .finish()
    }
}

// Three states that a OnceCell can be in, encoded into the lower bits of `queue` in
// the OnceCell structure.
const INCOMPLETE: usize = 0x0;
const RUNNING: usize = 0x1;
const COMPLETE: usize = 0x2;
const INCOMPLETE_PTR: *mut Waiter = INCOMPLETE as *mut Waiter;
const COMPLETE_PTR: *mut Waiter = COMPLETE as *mut Waiter;

// Mask to learn about the state. All other bits are the queue of waiters if
// this is in the RUNNING state.
const STATE_MASK: usize = 0x3;

/// Representation of a node in the linked list of waiters in the RUNNING state.
/// A waiters is stored on the stack of the waiting threads.
#[repr(align(4))] // Ensure the two lower bits are free to use as state bits.
struct Waiter {
    thread: Cell<Option<Thread>>,
    signaled: AtomicBool,
    next: *mut Waiter,
}

/// Drains and notifies the queue of waiters on drop.
struct Guard<'a> {
    queue: &'a AtomicPtr<Waiter>,
    new_queue: *mut Waiter,
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        let queue = self.queue.swap(self.new_queue, Ordering::AcqRel);

        let state = strict::addr(queue) & STATE_MASK;
        assert_eq!(state, RUNNING);

        unsafe {
            let mut waiter = strict::map_addr(queue, |q| q & !STATE_MASK);
            while !waiter.is_null() {
                let next = (*waiter).next;
                let thread = (*waiter).thread.take().unwrap();
                (*waiter).signaled.store(true, Ordering::Release);
                waiter = next;
                thread.unpark();
            }
        }
    }
}

// Corresponds to `std::sync::Once::call_inner`.
//
// Originally copied from std, but since modified to remove poisoning and to
// support wait.
//
// Note: this is intentionally monomorphic
#[inline(never)]
fn initialize_or_wait(queue: &AtomicPtr<Waiter>, mut init: Option<&mut dyn FnMut() -> bool>) {
    let mut curr_queue = queue.load(Ordering::Acquire);

    loop {
        let curr_state = strict::addr(curr_queue) & STATE_MASK;
        match (curr_state, &mut init) {
            (COMPLETE, _) => return,
            (INCOMPLETE, Some(init)) => {
                let exchange = queue.compare_exchange(
                    curr_queue,
                    strict::map_addr(curr_queue, |q| (q & !STATE_MASK) | RUNNING),
                    Ordering::Acquire,
                    Ordering::Acquire,
                );
                if let Err(new_queue) = exchange {
                    curr_queue = new_queue;
                    continue;
                }
                let mut guard = Guard {
                    queue,
                    new_queue: INCOMPLETE_PTR,
                };
                if init() {
                    guard.new_queue = COMPLETE_PTR;
                }
                return;
            }
            (INCOMPLETE, None) | (RUNNING, _) => {
                wait(queue, curr_queue);
                curr_queue = queue.load(Ordering::Acquire);
            }
            _ => debug_assert!(false),
        }
    }
}

fn wait(queue: &AtomicPtr<Waiter>, mut curr_queue: *mut Waiter) {
    let curr_state = strict::addr(curr_queue) & STATE_MASK;
    loop {
        let node = Waiter {
            thread: Cell::new(Some(thread::current())),
            signaled: AtomicBool::new(false),
            next: strict::map_addr(curr_queue, |q| q & !STATE_MASK),
        };
        let me = &node as *const Waiter as *mut Waiter;

        let exchange = queue.compare_exchange(
            curr_queue,
            strict::map_addr(me, |q| q | curr_state),
            Ordering::Release,
            Ordering::Relaxed,
        );
        if let Err(new_queue) = exchange {
            if strict::addr(new_queue) & STATE_MASK != curr_state {
                return;
            }
            curr_queue = new_queue;
            continue;
        }

        while !node.signaled.load(Ordering::Acquire) {
            thread::park();
        }
        break;
    }
}

unsafe fn unwrap_unchecked<T>(o: Option<T>) -> T {
    debug_assert!(o.is_some());
    match o {
        Some(value) => value,
        None => {
            unsafe { unreachable_unchecked() };
        }
    }
}

// Polyfill of strict provenance from https://crates.io/crates/sptr.
//
// Use free-standing function rather than a trait to keep things simple and
// avoid any potential conflicts with future stabile std API.
mod strict {
    #[must_use]
    #[inline]
    pub fn addr<T>(ptr: *mut T) -> usize
    where
        T: Sized,
    {
        // FIXME(strict_provenance_magic): I am magic and should be a compiler intrinsic.
        // SAFETY: Pointer-to-integer transmutes are valid (if you are okay with losing the
        // provenance).
        unsafe { core::mem::transmute(ptr) }
    }

    #[must_use]
    #[inline]
    pub fn with_addr<T>(ptr: *mut T, addr: usize) -> *mut T
    where
        T: Sized,
    {
        // FIXME(strict_provenance_magic): I am magic and should be a compiler intrinsic.
        //
        // In the mean-time, this operation is defined to be "as if" it was
        // a wrapping_offset, so we can emulate it as such. This should properly
        // restore pointer provenance even under today's compiler.
        let self_addr = self::addr(ptr) as isize;
        let dest_addr = addr as isize;
        let offset = dest_addr.wrapping_sub(self_addr);

        // This is the canonical desugarring of this operation,
        // but `pointer::cast` was only stabilized in 1.38.
        // self.cast::<u8>().wrapping_offset(offset).cast::<T>()
        (ptr as *mut u8).wrapping_offset(offset) as *mut T
    }

    #[must_use]
    #[inline]
    pub fn map_addr<T>(ptr: *mut T, f: impl FnOnce(usize) -> usize) -> *mut T
    where
        T: Sized,
    {
        self::with_addr(ptr, f(addr(ptr)))
    }
}

// These test are snatched from std as well.
#[cfg(test)]
mod tests {
    use std::{panic, sync::mpsc::channel, thread};

    use super::OnceCellInner;

    impl<T> OnceCellInner<T> {
        fn init(&self, f: impl FnOnce() -> T) {
            enum Void {}
            let _ = self.initialize(|| Ok::<T, Void>(f()));
        }
    }

    #[test]
    fn smoke_once() {
        static O: OnceCellInner<()> = OnceCellInner::new();
        let mut a = 0;
        O.init(|| a += 1);
        assert_eq!(a, 1);
        O.init(|| a += 1);
        assert_eq!(a, 1);
    }

    #[test]
    fn stampede_once() {
        static O: OnceCellInner<()> = OnceCellInner::new();
        static mut RUN: bool = false;

        let (tx, rx) = channel();
        for _ in 0..10 {
            let tx = tx.clone();
            thread::spawn(move || {
                for _ in 0..4 {
                    thread::yield_now()
                }
                unsafe {
                    O.init(|| {
                        assert!(!RUN);
                        RUN = true;
                    });
                    assert!(RUN);
                }
                tx.send(()).unwrap();
            });
        }

        unsafe {
            O.init(|| {
                assert!(!RUN);
                RUN = true;
            });
            assert!(RUN);
        }

        for _ in 0..10 {
            rx.recv().unwrap();
        }
    }

    #[test]
    fn poison_bad() {
        static O: OnceCellInner<()> = OnceCellInner::new();

        // poison the once
        let t = panic::catch_unwind(|| {
            O.init(|| panic!());
        });
        assert!(t.is_err());

        // we can subvert poisoning, however
        let mut called = false;
        O.init(|| {
            called = true;
        });
        assert!(called);

        // once any success happens, we stop propagating the poison
        O.init(|| {});
    }

    #[test]
    fn wait_for_force_to_finish() {
        static O: OnceCellInner<()> = OnceCellInner::new();

        // poison the once
        let t = panic::catch_unwind(|| {
            O.init(|| panic!());
        });
        assert!(t.is_err());

        // make sure someone's waiting inside the once via a force
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();
        let t1 = thread::spawn(move || {
            O.init(|| {
                tx1.send(()).unwrap();
                rx2.recv().unwrap();
            });
        });

        rx1.recv().unwrap();

        // put another waiter on the once
        let t2 = thread::spawn(|| {
            let mut called = false;
            O.init(|| {
                called = true;
            });
            assert!(!called);
        });

        tx2.send(()).unwrap();

        assert!(t1.join().is_ok());
        assert!(t2.join().is_ok());
    }

    #[test]
    #[cfg(target_pointer_width = "64")]
    fn test_size() {
        use std::mem::size_of;

        assert_eq!(size_of::<OnceCellInner<u32>>(), 4 * size_of::<u32>());
    }
}
