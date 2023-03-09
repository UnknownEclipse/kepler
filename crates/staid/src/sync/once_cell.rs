use core::{cell::UnsafeCell, convert::Infallible, fmt::Debug};

use super::once::Once;

pub struct OnceCell<T> {
    once: Once,
    value: UnsafeCell<Option<T>>,
}

impl<T> OnceCell<T> {
    pub const fn new() -> Self {
        OnceCell {
            once: Once::new(),
            value: UnsafeCell::new(None),
        }
    }

    pub const fn with_value(value: T) -> Self {
        Self {
            once: Once::completed(),
            value: UnsafeCell::new(Some(value)),
        }
    }

    pub fn get(&self) -> Option<&T> {
        if self.once.is_completed() {
            Some(unsafe { self.get_unchecked() })
        } else {
            None
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.value.get_mut().as_mut()
    }

    pub fn into_inner(self) -> Option<T> {
        self.value.into_inner()
    }

    /// # Safety
    /// This `OnceCell` must be fully initialized.
    pub unsafe fn get_unchecked(&self) -> &T {
        (*self.value.get()).as_ref().unwrap_unchecked()
    }

    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        self.get_or_try_init(|| Ok::<_, Infallible>(f())).unwrap()
    }

    pub fn get_or_try_init<F, E>(&self, f: F) -> Result<&T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        if let Some(value) = self.get() {
            Ok(value)
        } else {
            self.get_or_try_init_slow(f)
        }
    }

    fn get_or_try_init_slow<F, E>(&self, f: F) -> Result<&T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let mut f = Some(f);

        self.once
            .try_call_once(|| unsafe {
                let f = f.take().unwrap_unchecked();
                match f() {
                    Ok(value) => {
                        *self.value.get() = Some(value);
                        Ok(())
                    }
                    Err(err) => Err(err),
                }
            })
            .map(|_| unsafe { self.get_unchecked() })
    }

    pub fn wait(&self) -> &T {
        self.once.wait();
        unsafe { self.get_unchecked() }
    }
}

unsafe impl<T> Send for OnceCell<T> where T: Send {}
unsafe impl<T> Sync for OnceCell<T> where T: Sync + Send {}

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

impl<T> Default for OnceCell<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for OnceCell<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        self.get()
            .cloned()
            .map(Self::with_value)
            .unwrap_or_default()
    }
}
