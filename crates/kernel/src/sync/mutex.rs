use core::{
    hint,
    sync::atomic::{AtomicU32, Ordering},
};

use lock_api::GuardSend;

use super::futex::{wait, wake_one};

pub type Mutex<T> = lock_api::Mutex<RawMutex, T>;
pub type MutexGuard<'a, T> = lock_api::MutexGuard<'a, RawMutex, T>;

#[derive(Debug)]
pub struct RawMutex {
    state: AtomicU32,
}

impl RawMutex {
    #[cold]
    fn lock_contended(&self) {
        let mut state = self.spin();

        if state == 0 {
            match self
                .state
                .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
            {
                Ok(_) => return,
                Err(s) => state = s,
            }
        }

        loop {
            if state != 2 && self.state.swap(2, Ordering::Acquire) == 0 {
                return;
            }
            wait(&self.state, 2);
            state = self.spin();
        }
    }

    #[cold]
    fn wake(&self) {
        wake_one(&self.state);
    }

    fn spin(&self) -> u32 {
        let mut spin = 100;
        loop {
            let state = self.state.load(Ordering::Relaxed);
            if state != 2 || spin == 0 {
                return state;
            }
            hint::spin_loop();
            spin -= 1;
        }
    }
}

unsafe impl lock_api::RawMutex for RawMutex {
    type GuardMarker = GuardSend;

    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = Self {
        state: AtomicU32::new(0),
    };

    #[inline]
    fn try_lock(&self) -> bool {
        self.state
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
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
        if self.state.swap(0, Ordering::Release) == 2 {
            self.wake();
        }
    }
}
