use core::{
    cell::Cell,
    fmt,
    ops::{Deref, DerefMut},
};

use crate::OnceCell;

pub struct Lazy<T, F = fn() -> T> {
    cell: OnceCell<T>,
    init: Cell<Option<F>>,
}

impl<T, F> Lazy<T, F> {
    pub const fn new(init: F) -> Self {
        Self {
            cell: OnceCell::new(),
            init: Cell::new(Some(init)),
        }
    }

    pub fn into_value(this: Self) -> Result<T, F> {
        this.cell
            .into_inner()
            .ok_or_else(|| this.init.into_inner().expect("invalid lazy state"))
    }
}

impl<T, F> Lazy<T, F>
where
    F: FnOnce() -> T,
{
    pub fn force(this: &Self) -> &T {
        this.cell.get_or_init(|| {
            let init = this.init.take().unwrap();
            init()
        })
    }

    pub fn force_mut(this: &mut Self) -> &mut T {
        Self::force(this);
        Self::get_mut(this).unwrap_or_else(|| unreachable!())
    }

    pub fn get(this: &Self) -> Option<&T> {
        this.cell.get()
    }

    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        this.cell.get_mut()
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

impl<T: fmt::Debug, F> fmt::Debug for Lazy<T, F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Lazy")
            .field("cell", &self.cell)
            .field("init", &"..")
            .finish()
    }
}
