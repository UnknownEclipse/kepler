use hal::interrupts::{self, NoInterrupts};
use lock_api::RawMutex;

pub struct IrqMutex<R, T> {
    inner: lock_api::Mutex<R, T>,
}

impl<R, T> IrqMutex<R, T>
where
    R: RawMutex,
{
    pub const fn new(value: T) -> Self {
        Self {
            inner: lock_api::Mutex::new(value),
        }
    }

    pub fn lock<F, U>(&self, f: F) -> U
    where
        F: FnOnce(&mut T, &mut NoInterrupts) -> U,
    {
        interrupts::without(|token| {
            let mut guard = self.inner.lock();
            f(&mut guard, token)
        })
    }
}

pub type SpinIrqMutex<T> = IrqMutex<spin::Mutex<()>, T>;
