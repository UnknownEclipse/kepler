use alloc::vec::Vec;
use core::{
    mem,
    sync::atomic::{AtomicBool, Ordering},
    task::Waker,
};

use spin::mutex::SpinMutex;

use crate::thread::{self, park, Thread};

#[derive(Debug)]
pub struct OneShotEvent {
    waiters: SpinMutex<Vec<Thread>>,
    is_set: AtomicBool,
}

impl OneShotEvent {
    #[inline]
    pub fn new() -> Self {
        Self {
            waiters: Default::default(),
            is_set: AtomicBool::new(false),
        }
    }

    #[inline]
    pub fn wait(&self) {
        if !self.is_set() {
            self.wait_slow();
        }
    }

    #[cold]
    fn wait_slow(&self) {
        self.waiters.lock().push(thread::current());

        while !self.is_set() {
            park();
        }
    }

    #[inline]
    pub fn is_set(&self) -> bool {
        self.is_set.load(Ordering::Acquire)
    }

    pub fn notify(&self) {
        self.is_set.store(true, Ordering::Release);
        let queue = mem::take(&mut *self.waiters.lock());
        for waker in queue {
            waker.unpark();
        }
    }
}

impl Default for OneShotEvent {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
