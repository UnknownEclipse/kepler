use core::{marker::PhantomData, ptr};

use crate::access::{Read, ReadWrite, Write};

#[repr(transparent)]
#[derive(Debug, Default)]
pub struct Volatile<T, A = ReadWrite> {
    value: T,
    _access: PhantomData<A>,
}

impl<T, A> Volatile<T, A> {
    pub const fn new(value: T) -> Self {
        Self {
            value,
            _access: PhantomData,
        }
    }

    pub const fn from_ref(value: &T) -> &Self {
        let ptr: *const T = value;
        unsafe { &*ptr.cast() }
    }

    pub const fn from_mut(value: &mut T) -> &mut Self {
        let ptr: *mut T = value;
        unsafe { &mut *ptr.cast() }
    }

    pub fn try_map<F, U, E>(&self, f: F) -> Result<&Volatile<U, A>, E>
    where
        F: FnOnce(&T) -> Result<&U, E>,
    {
        f(&self.value).map(Volatile::from_ref)
    }

    pub fn try_map_mut<F, U, E>(&mut self, f: F) -> Result<&mut Volatile<U, A>, E>
    where
        F: FnOnce(&mut T) -> Result<&mut U, E>,
    {
        f(&mut self.value).map(Volatile::from_mut)
    }

    pub fn map<F, U>(&self, f: F) -> &Volatile<U, A>
    where
        F: FnOnce(&T) -> &U,
    {
        Volatile::from_ref(f(&self.value))
    }

    pub fn map_mut<F, U>(&mut self, f: F) -> &mut Volatile<U, A>
    where
        F: FnOnce(&mut T) -> &mut U,
    {
        Volatile::from_mut(f(&mut self.value))
    }
}

impl<T, A> Volatile<T, A>
where
    T: Copy,
    A: Read,
{
    pub fn read(&self) -> T {
        read_volatile(&self.value)
    }
}

impl<T, A> Volatile<T, A>
where
    T: Copy,
    A: Write,
{
    pub fn write(&mut self, value: T) {
        write(&mut self.value, value);
    }
}

pub fn read_volatile<T>(value: &T) -> T
where
    T: Copy,
{
    unsafe { ptr::read_volatile(value) }
}

pub fn write<T>(dst: &mut T, value: T)
where
    T: Copy,
{
    unsafe { ptr::write_volatile(dst, value) };
}
