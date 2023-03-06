use core::{
    cell::Cell,
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, Ordering},
    task::Waker,
};

use lagrange::thread;

#[derive(Debug)]
struct MutexCore {
    state: AtomicPtr<Waiter>,
}

impl MutexCore {
    #[inline]
    pub fn try_lock(&self) -> bool {
        self.state
            .compare_exchange(unlocked(), locked(), Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    #[inline]
    pub fn lock(&self) {
        if !self.try_lock() {
            self.lock_contended(thread::current().into_waker());
        }
    }

    #[cold]
    pub fn lock_contended(&self, waker: Waker) {
        let waiter = Waiter {
            last: AtomicPtr::new(ptr::null_mut()),
            next: AtomicPtr::new(ptr::null_mut()),
            prev: Cell::new(None),
            waker: Cell::new(Some(waker)),
        };
        let waiter_ptr = &waiter as *const Waiter as *mut Waiter;

        loop {
            let state = self.state.load(Ordering::Acquire);

            if state == unlocked() {
                if self.try_lock() {
                    return;
                }
                continue;
            }

            let queue = wait_queue(state);

            let new_state = if let Some(head) = queue {
                todo!()
            } else {
                waiter.last.store(waiter_ptr, Ordering::Release);
                waiter.prev.set(None);
                waiter.next.store(ptr::null_mut(), Ordering::Release);
                waiter_ptr.map_addr(|addr| addr | 1)
            };
        }
    }

    unsafe fn unlock(&self) {
        let queue = self.state.load(Ordering::Acquire);
        let queue = wait_queue(queue);
        self.state.store(unlocked(), Ordering::Release);

        // There may be a thread that barges here before we do the waking and re-queuing
        // which is fine as we don't promise fairness.

        if let Some(head) = queue {
            self.unlock_slow(head);
        }
    }

    /// Pop the first in the queue, unpark it, then re-queue the rest of the waiters
    #[cold]
    unsafe fn unlock_slow(&self, head: NonNull<Waiter>) {
        let head = head.as_ref();
        let last = head.last.load(Ordering::Relaxed);
        let rest = head.next.load(Ordering::Acquire);

        if let Some(waker) = head.waker.take() {
            waker.wake();
        }

        let rest = NonNull::new(rest);
        if let Some(rest) = rest {
            rest.as_ref().last.store(last, Ordering::Release);
            self.push_wait_list(rest);
        }
    }

    #[cold]
    unsafe fn push_wait_list(&self, head: NonNull<Waiter>) {
        todo!()
    }
}

struct Waiter {
    prev: Cell<Option<NonNull<Waiter>>>,
    next: AtomicPtr<Waiter>,
    last: AtomicPtr<Waiter>,
    waker: Cell<Option<Waker>>,
}

#[derive(Debug, Clone, Copy)]
enum State {
    Unlocked,
    Locked,
    LockedContended(NonNull<Waiter>),
}

impl State {
    pub fn from_raw(raw: *const Waiter) -> Self {
        todo!()
    }

    pub fn to_raw(self) -> *mut Waiter {
        todo!()
    }
}

fn from_raw(raw: *const Waiter) -> State {
    State::from_raw(raw)
}

fn unlocked() -> *mut Waiter {
    State::Unlocked.to_raw()
}

fn locked() -> *mut Waiter {
    State::Locked.to_raw()
}

fn wait_queue(raw: *mut Waiter) -> Option<NonNull<Waiter>> {
    todo!()
}

fn is_locked(state: *const Waiter) {
    todo!()
}
