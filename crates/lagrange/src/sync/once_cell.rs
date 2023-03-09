// This i0ied verbatim from the once_cell std implementation [here](https://github.com/matklad/once_cell/blob/master/src/imp_std.rs),
// but adapter to use the kernel task api instead.
// There's a lot of scary concurrent code in this module, but it is copied from
// `std::sync::Once` with two changes:
//   * no poisoning
//   * init function can fail

use core::{cell::UnsafeCell, fmt::Debug, mem};

use super::Once;

pub struct OnceCell<T> {
    once: Once,
    value: UnsafeCell<Option<T>>,
}

impl<T> OnceCell<T> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            once: Once::new(),
            value: UnsafeCell::new(None),
        }
    }

    #[inline]
    pub const fn with_value(value: T) -> Self {
        Self {
            once: Once::new(),
            value: UnsafeCell::new(Some(value)),
        }
    }

    #[inline]
    pub fn get(&self) -> Option<&T> {
        if self.once.is_completed() {
            Some(unsafe { self.get_unchecked() })
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.value.get_mut().as_mut()
    }

    #[inline]
    pub fn into_inner(self) -> Option<T> {
        self.value.into_inner()
    }

    /// # Safety
    /// 1. This `OnceCell` must be initialized.
    #[inline]
    pub unsafe fn get_unchecked(&self) -> &T {
        (*self.value.get()).as_ref().unwrap_unchecked()
    }

    pub fn wait(&self) -> &T {
        self.once.wait();
        debug_assert!(self.once.is_completed());
        unsafe { self.get_unchecked() }
    }

    pub fn try_insert(&self, value: T) -> Result<&T, (&T, T)> {
        let mut value = Some(value);
        let res = self.get_or_init(|| unsafe { value.take().unwrap_unchecked() });
        match value {
            None => Ok(res),
            Some(value) => Err((res, value)),
        }
    }

    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        enum Void {}
        match self.get_or_try_init(|| Ok::<_, Void>(f())) {
            Ok(val) => val,
            Err(void) => match void {},
        }
    }

    pub fn get_or_try_init<F, E>(&self, f: F) -> Result<&T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        if let Some(value) = self.get() {
            return Ok(value);
        }

        self.once
            .try_call_once(|| -> Result<(), E> {
                let value = f()?;
                let slot = self.value.get();
                unsafe { *slot = Some(value) };
                Ok(())
            })
            .map(|_| unsafe { self.get_unchecked() })
    }

    pub fn take(&mut self) -> Option<T> {
        mem::take(self).into_inner()
    }
}

unsafe impl<T> Sync for OnceCell<T> where T: Send + Sync {}
unsafe impl<T> Send for OnceCell<T> where T: Send {}

impl<T> Default for OnceCell<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Debug for OnceCell<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("OnceCell")
            .field("value", &self.get())
            .finish()
    }
}

impl<T> Clone for OnceCell<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        self.get()
            .cloned()
            .map(OnceCell::with_value)
            .unwrap_or_default()
    }
}
