use core::ops::Deref;

#[repr(transparent)]
#[derive(Debug)]
pub struct UnsafeMut<T: ?Sized>(T);

impl<T> UnsafeMut<T> {
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    /// # Safety
    /// The safety requirements from the wrapped object hold here.
    pub unsafe fn into_inner(self) -> T {
        self.0
    }
}

impl<T: ?Sized> UnsafeMut<T> {
    pub const fn from_ref(value: &T) -> &Self {
        unsafe { &*(value as *const T as *const Self) }
    }

    pub const fn from_mut(value: &mut T) -> &mut Self {
        unsafe { &mut *(value as *mut T as *mut Self) }
    }

    pub const fn get(&self) -> &T {
        &self.0
    }

    /// # Safety
    /// The safety requirements from the wrapped object hold here.
    pub const unsafe fn get_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: ?Sized> Deref for UnsafeMut<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}
