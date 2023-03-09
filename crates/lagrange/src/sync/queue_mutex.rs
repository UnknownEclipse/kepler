use alloc::collections::VecDeque;
use core::{
    hint,
    sync::atomic::{AtomicU8, Ordering},
};

use hal::interrupts;
use lock_api::{GuardSend, RawMutex};
use spin::mutex::SpinMutex;

use crate::thread::{self, park, Thread};

const UNLOCKED: u8 = 0;
const LOCKED: u8 = 1;
const CONTENDED: u8 = 2;

pub type IqMutex<T> = lock_api::Mutex<RawIqMutex, T>;

/// A mutex that manages it's own thread queue.
///
/// Currently this is a bad first implementation that is slow and too fat, but it
/// will be improved later.
#[derive(Debug)]
pub struct RawIqMutex {
    state: AtomicU8,
    queue: SpinMutex<VecDeque<Thread>>,
}

impl RawIqMutex {
    #[cold]
    fn lock_contended(&self) {
        let mut state = self.spin();

        if state == 0 {
            match self.state.compare_exchange(
                UNLOCKED,
                LOCKED,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => return,
                Err(s) => state = s,
            }
        }

        loop {
            if state != CONTENDED && self.state.swap(CONTENDED, Ordering::Acquire) == UNLOCKED {
                return;
            }

            interrupts::without(|_| {
                self.queue.lock().push_back(thread::current());
                park();
            });

            state = self.spin();
        }
    }

    #[cold]
    fn wake(&self) {
        let t = interrupts::without(|_| self.queue.lock().pop_front());
        if let Some(t) = t {
            t.unpark();
        }
    }

    fn spin(&self) -> u8 {
        let mut spin = 100;
        loop {
            let state = self.state.load(Ordering::Relaxed);
            if state != LOCKED || spin == 0 {
                return state;
            }
            hint::spin_loop();
            spin -= 1;
        }
    }
}

unsafe impl RawMutex for RawIqMutex {
    type GuardMarker = GuardSend;

    const INIT: Self = Self {
        state: AtomicU8::new(UNLOCKED),
        queue: SpinMutex::new(VecDeque::new()),
    };

    #[inline]
    fn try_lock(&self) -> bool {
        self.state
            .compare_exchange(UNLOCKED, LOCKED, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    #[inline]
    fn lock(&self) {
        if !self.try_lock() {
            self.lock_contended();
        }
    }

    #[inline]
    unsafe fn unlock(&self) {
        if self.state.swap(UNLOCKED, Ordering::Release) == CONTENDED {
            self.wake();
        }
    }
}
