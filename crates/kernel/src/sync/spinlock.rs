use core::ops::{Deref, DerefMut};

use hal::interrupts::{self, WithoutInterrupts};
use spin::mutex::{SpinMutex, SpinMutexGuard};

#[derive(Debug)]
pub struct SpinLock<T>(SpinMutex<T>);

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self(SpinMutex::new(value))
    }

    pub fn lock<'a>(&'a self, _g: &'a WithoutInterrupts) -> SpinLockGuard<'a, T> {
        SpinLockGuard(self.0.lock())
    }

    pub fn try_lock<'a>(&'a self, _g: &'a WithoutInterrupts) -> Option<SpinLockGuard<'a, T>> {
        self.0.try_lock().map(SpinLockGuard)
    }

    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        interrupts::without(|g| {
            let mut guard = self.lock(g);
            f(&mut guard)
        })
    }
}

#[derive(Debug)]
pub struct SpinLockGuard<'a, T>(SpinMutexGuard<'a, T>);

impl<'a, T> Deref for SpinLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T> DerefMut for SpinLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
