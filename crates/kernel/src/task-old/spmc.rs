use alloc::sync::Arc;
use core::{
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, AtomicU32, Ordering},
};

use super::thread::Thread;

pub struct Local {
    queue: Arc<UnsafeQueue>,
}

impl Local {
    pub fn pop(&self) -> Option<Thread> {
        unsafe {
            let ptr = self.queue.pop_raw();
            to_thread(ptr)
        }
    }

    pub fn push(&mut self, thread: Thread) -> Result<(), Thread> {
        let raw = thread.into_raw();

        unsafe {
            if self.queue.push_raw(raw.as_ptr()) {
                Ok(())
            } else {
                Err(Thread::from_raw(raw))
            }
        }
    }
}

pub struct Steal {
    queue: Arc<UnsafeQueue>,
}

impl Steal {
    pub fn steal_into(&self, into: &mut Local) -> Option<Thread> {
        unsafe {
            let ptr = self.queue.steal_raw(&into.queue);
            to_thread(ptr)
        }
    }
}
unsafe fn to_thread(ptr: *mut ()) -> Option<Thread> {
    NonNull::new(ptr).map(|raw| Thread::from_raw(raw))
}

#[derive(Debug)]
pub struct UnsafeQueue {
    head: AtomicU32,
    tail: AtomicU32,
    buffer: [AtomicPtr<()>; SIZE],
}

const SIZE: usize = 256;
const MASK: u32 = (SIZE - 1) as u32;

impl UnsafeQueue {
    pub fn new() -> Self {
        #[allow(clippy::declare_interior_mutable_const)]
        const INIT: AtomicPtr<()> = AtomicPtr::new(ptr::null_mut());

        UnsafeQueue {
            head: AtomicU32::new(0),
            tail: AtomicU32::new(0),
            buffer: [INIT; SIZE],
        }
    }

    pub fn pop(&self) -> Option<Thread> {
        unsafe {
            let ptr = self.pop_raw();
            to_thread(ptr)
        }
    }

    pub unsafe fn push(&self, thread: Thread) -> Result<(), Thread> {
        let raw = thread.into_raw();

        if self.push_raw(raw.as_ptr()) {
            Ok(())
        } else {
            Err(Thread::from_raw(raw))
        }
    }

    pub unsafe fn steal_into(&self, into: &UnsafeQueue) -> Option<Thread> {
        let ptr = self.steal_raw(into);
        to_thread(ptr)
    }

    fn pop_raw(&self) -> *mut () {
        let mut head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);

        while head != tail {
            match self.head.compare_exchange_weak(
                head,
                head.wrapping_add(1),
                Ordering::Acquire,
                Ordering::Acquire,
            ) {
                Ok(_) => return self.buffer[(head & MASK) as usize].load(Ordering::Relaxed),
                Err(h) => {
                    head = h;
                }
            }
        }
        ptr::null_mut()
    }

    unsafe fn push_raw(&self, ptr: *mut ()) -> bool {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);

        if tail.wrapping_sub(head) >= (SIZE as u32) {
            return false;
        }

        self.buffer[(tail & MASK) as usize].store(ptr, Ordering::Relaxed);
        self.tail.store(tail.wrapping_add(1), Ordering::Release);
        true
    }

    fn steal_raw(&self, into: &UnsafeQueue) -> *mut () {
        loop {
            let head = self.head.load(Ordering::Acquire);
            let tail = self.tail.load(Ordering::Acquire);

            if tail.wrapping_sub(head) > (SIZE as u32) {
                continue;
            }
            if tail == head {
                return ptr::null_mut();
            }

            let half = tail.wrapping_sub(head) - (tail.wrapping_sub(head) / 2);
            for i in 0..half {
                let v = self.buffer[index(head.wrapping_add(i))].load(Ordering::Relaxed);

                let dst_index = into.tail.load(Ordering::Relaxed).wrapping_add(i);
                into.buffer[index(dst_index)].store(v, Ordering::Relaxed);
            }

            if self
                .head
                .compare_exchange(
                    head,
                    head.wrapping_add(half),
                    Ordering::AcqRel,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                let new_tail = into.tail.load(Ordering::Relaxed).wrapping_add(half);
                into.tail.store(new_tail.wrapping_sub(1), Ordering::Release);
                return into.buffer[index(new_tail)].load(Ordering::Relaxed);
            }
        }
    }
}

fn index(v: u32) -> usize {
    (v & MASK) as usize
}
enum StealRaw {
    Empty,
}
