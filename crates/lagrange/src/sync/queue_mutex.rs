use core::{cell::Cell, ptr::NonNull, sync::atomic::AtomicPtr};

use crate::thread::Thread;

struct QueueMutex {
    state: AtomicPtr<Waiter>,
}

impl QueueMutex {
    unsafe fn lock_queue(&self) {}

    unsafe fn unlock_queue(&self) {}
}

struct Waiter {
    thread: Cell<Option<Thread>>,
    next: Cell<Option<NonNull<Waiter>>>,
}
