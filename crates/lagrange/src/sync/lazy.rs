use core::{
    cell::Cell,
    ops::{Deref, DerefMut},
};

use super::OnceCell;

pub struct Lazy<T, F = fn() -> T> {
    value: OnceCell<T>,
    init: Cell<Option<F>>,
}

impl<T, F> Lazy<T, F> {
    pub const fn new(init: F) -> Self {
        Self {
            value: OnceCell::new(),
            init: Cell::new(Some(init)),
        }
    }

    pub fn get(this: &Self) -> Option<&T> {
        this.value.get()
    }

    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        this.value.get_mut()
    }
}

impl<T, F> Lazy<T, F>
where
    F: FnOnce() -> T,
{
    pub fn force(this: &Self) -> &T {
        this.value.get_or_init(|| {
            let f = unsafe { this.init.take().unwrap_unchecked() };
            f()
        })
    }

    pub fn force_mut(this: &mut Self) -> &mut T {
        Self::force(this);
        unsafe { Self::get_mut(this).unwrap_unchecked() }
    }
}

impl<T, F> Deref for Lazy<T, F>
where
    F: FnOnce() -> T,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        Self::force(self)
    }
}

impl<T, F> DerefMut for Lazy<T, F>
where
    F: FnOnce() -> T,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        Self::force_mut(self)
    }
}

impl<T> Default for Lazy<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(Default::default)
    }
}
