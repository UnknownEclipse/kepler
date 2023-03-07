use core::{
    cell::Cell,
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::thread::{self, Thread};

#[derive(Debug)]
struct QueueMutex {
    state: AtomicPtr<Waiter>,
}

const UNLOCKED: *mut Waiter = 0 as *mut Waiter;
const LOCKED: *mut Waiter = 1 as *mut Waiter;

impl QueueMutex {
    pub const fn new() -> Self {
        Self {
            state: AtomicPtr::new(UNLOCKED),
        }
    }

    #[inline]
    pub fn is_locked(&self) -> bool {
        is_locked(self.state.load(Ordering::Acquire))
    }

    #[inline]
    pub fn try_lock(&self) -> bool {
        self.state
            .compare_exchange(UNLOCKED, LOCKED, Ordering::AcqRel, Ordering::Relaxed)
            .is_ok()
    }

    #[inline]
    pub fn lock(&self) {
        if !self.try_lock() {
            self.lock_slow();
        }
    }

    #[cold]
    fn lock_slow(&self) {
        // TODO: Spin for a bit before parking
        loop {
            let Err(state) = self.state.compare_exchange_weak(UNLOCKED , LOCKED , Ordering::Acquire, Ordering::Relaxed) else {
                return;
            };
            let waiter = Waiter {
                last: AtomicPtr::new(ptr::null_mut()),
                next: AtomicPtr::new(ptr::null_mut()),
                prev: Cell::new(None),
                thread: Cell::new(Some(thread::current())),
            };
            let waiter_ptr: *const Waiter = &waiter;
            let queue_head = queue_head(state);

            let last = queue_head.map(|addr| unsafe { addr.as_ref().last.load(Ordering::Acquire) });

            waiter
                .last
                .store(last.unwrap_or(ptr::null_mut()), Ordering::Release);

            let new_state = waiter_ptr.map_addr(|addr| addr | 1).cast_mut();
            if self
                .state
                .compare_exchange_weak(state, new_state, Ordering::AcqRel, Ordering::Relaxed)
                .is_err()
            {
                continue;
            }
        }
        todo!()
    }

    pub unsafe fn unlock(&self) {
        let state = self.state.swap(UNLOCKED, Ordering::Release);

        if let Some(head) = queue_head(state) {
            self.unlock_slow(head);
        }
    }

    #[inline]
    unsafe fn unlock_slow(&self, queue_head: NonNull<Waiter>) {
        todo!()
    }
}

struct Waiter {
    last: AtomicPtr<Waiter>,
    next: AtomicPtr<Waiter>,
    prev: Cell<Option<NonNull<Waiter>>>,
    thread: Cell<Option<Thread>>,
}

fn is_locked(state: *mut Waiter) -> bool {
    (state as usize) & 1 != 0
}

fn queue_head(state: *mut Waiter) -> Option<NonNull<Waiter>> {
    todo!()
}
