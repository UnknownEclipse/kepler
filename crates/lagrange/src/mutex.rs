use core::{
    cell::Cell,
    marker::PhantomPinned,
    pin::Pin,
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, Ordering},
};

use lock_api::GuardSend;

use crate::thread::Thread;

/// An adaptive sleeping mutex.
pub struct RawMutex {
    /// 0b0: Unlocked
    /// 0b1: Locked
    state: AtomicPtr<Waiter>,
}

impl RawMutex {
    #[cold]
    fn lock_contended(&self) {}

    fn push_waiter(&self, waiter: Pin<&Waiter>) {
        loop {
            let state = self.state.load(Ordering::Relaxed);
        }
    }
}

const UNLOCKED: *mut Waiter = 0 as *mut Waiter;
const LOCKED: *mut Waiter = 1 as *mut Waiter;

unsafe impl lock_api::RawMutex for RawMutex {
    type GuardMarker = GuardSend;

    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = RawMutex {
        state: AtomicPtr::new(ptr::null_mut()),
    };

    #[inline]
    fn lock(&self) {
        if !self.try_lock() {
            self.lock_contended();
        }
    }

    #[inline]
    fn try_lock(&self) -> bool {
        self.state
            .compare_exchange(UNLOCKED, LOCKED, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    unsafe fn unlock(&self) {
        todo!()
    }
}

#[derive(Debug)]
struct Waiter {
    next: Cell<Option<NonNull<Waiter>>>,
    prev: AtomicPtr<Waiter>,
    tail: AtomicPtr<Waiter>,
    thread: Thread,
    _pin: PhantomPinned,
}
