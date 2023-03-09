use core::{
    cell::UnsafeCell,
    fmt::Debug,
    sync::atomic::{AtomicU8, Ordering},
};

pub use imp::{ExceptionEntry, InterruptEntry, InterruptTable, StackFrame};

use crate::imp::interrupts as imp;

#[non_exhaustive]
#[derive(Debug)]
pub struct WithoutInterrupts;

impl WithoutInterrupts {
    /// # Safety
    /// 1. This method allows getting an interrupt guard, potentially without them
    /// actually being disabled. Any code that depends on that guard for safety will
    /// become unsafe in this case.
    pub unsafe fn unprotected() -> Self {
        Self
    }
}

#[inline]
pub unsafe fn enable() {
    imp::enable();
}

#[inline]
pub unsafe fn enable_and_wait() {
    imp::enable_and_wait();
}

#[inline]
pub unsafe fn disable() {
    imp::disable();
}

#[inline]
pub unsafe fn are_enabled() -> bool {
    imp::are_enabled()
}

#[inline]
pub unsafe fn wait() {
    imp::wait();
}

#[inline]
pub fn without<F, T>(f: F) -> T
where
    F: FnOnce(&mut WithoutInterrupts) -> T,
{
    unsafe {
        let were_enabled = are_enabled();
        if were_enabled {
            disable();
        }
        let mut token = WithoutInterrupts;
        let result = f(&mut token);
        if were_enabled {
            enable();
        }
        result
    }
}

pub trait InterruptHandler {
    fn handle(stack_frame: &mut StackFrame);
}

pub trait ExceptionHandler {
    type Output;
    type Error;

    fn handle(stack_frame: &mut StackFrame, error: Self::Error) -> Self::Output;
}

/// An interrupt-aware spinlock.
pub struct SpinLock<T> {
    value: UnsafeCell<T>,
    raw: RawSpinLock,
}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            raw: RawSpinLock::new(),
        }
    }

    pub fn with<F, U>(&self, f: F) -> U
    where
        F: FnOnce(&mut T) -> U,
    {
        unsafe {
            self.raw.lock();
            let result = f(&mut *self.value.get());
            self.raw.unlock();
            result
        }
    }
}

unsafe impl<T> Send for SpinLock<T> where T: Send {}
unsafe impl<T> Sync for SpinLock<T> where T: Send {}

impl<T> Debug for SpinLock<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SpinLock").finish_non_exhaustive()
    }
}

/// The core impl of a spin lock that disables interrupts while it is locked. This
/// prevents issues like a thread spinning on a thread that is parked by the scheduler.
struct RawSpinLock {
    state: AtomicU8,
}

impl RawSpinLock {
    pub const fn new() -> Self {
        Self {
            state: AtomicU8::new(0),
        }
    }

    pub unsafe fn try_lock(&self) -> bool {
        let were_enabled = are_enabled();
        if were_enabled {
            disable();
        }
        let state = (u8::from(were_enabled) << 1) | 1;

        let ok = self
            .state
            .compare_exchange(0, state, Ordering::Acquire, Ordering::Relaxed)
            .is_ok();

        if !ok && were_enabled {
            enable();
        }
        ok
    }

    pub unsafe fn lock(&self) {
        let were_enabled = are_enabled();
        if were_enabled {
            disable();
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

    pub unsafe fn unlock(&self) {
        let state = self.state.swap(0, Ordering::Release);
        let were_enabled = state & 0b10 != 0;
        if were_enabled {
            enable();
        }
    }
}
