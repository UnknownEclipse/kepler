use core::{
    fmt::{Debug, Pointer},
    ops::Deref,
    ptr::NonNull,
};

pub struct UnsafeRef<T>(NonNull<T>);

impl<T> UnsafeRef<T> {
    pub const fn from_static(value: &'static T) -> Self {
        unsafe { Self::from_ptr(value) }
    }

    pub const unsafe fn from_ptr(ptr: *const T) -> Self {
        Self::from_raw(unsafe { NonNull::new_unchecked(ptr.cast_mut()) })
    }

    pub const unsafe fn from_raw(ptr: NonNull<T>) -> Self {
        Self(ptr)
    }

    pub const fn to_raw(&self) -> NonNull<T> {
        self.0
    }
}

impl<T> Deref for UnsafeRef<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref() }
    }
}

impl<T> Debug for UnsafeRef<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.deref().fmt(f)
    }
}

impl<T> Clone for UnsafeRef<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for UnsafeRef<T> {}
