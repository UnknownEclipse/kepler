use core::sync::atomic::{AtomicU8, Ordering};

use hal::interrupts::{self, disable};

pub struct RawSpinMutex {
    state: AtomicU8,
}

struct SpinMutexCore {
    state: AtomicU8,
}

impl SpinMutexCore {
    pub unsafe fn lock(&self) {
        let were_enabled = interrupts::are_enabled();
        if were_enabled {
            interrupts::disable();
        }
        let state = (u8::from(were_enabled) << 1) | 1;

        if self
            .state
            .compare_exchange(0, state, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            self.lock_contended(state);
        }
    }

    #[cold]
    unsafe fn lock_contended(&self, state: u8) {
        while self
            .state
            .compare_exchange_weak(0, state, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.state.load(Ordering::Acquire) != 0 {
                core::hint::spin_loop();
            }
        }
    }

    pub unsafe fn unlock(&self) {}
}
