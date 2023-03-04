use core::marker::PhantomData;

use hal_core::access::{Read, ReadWrite, Write};

use crate::intrin::{
    port_read_u16, port_read_u32, port_read_u8, port_write_u16, port_write_u32, port_write_u8,
};

#[derive(Debug)]
pub struct Port<T, A = ReadWrite> {
    port: u16,
    _access: PhantomData<A>,
    _type: PhantomData<T>,
}

impl<T, A> Port<T, A> {
    pub const fn new(port: u16) -> Self {
        Port {
            port,
            _access: PhantomData,
            _type: PhantomData,
        }
    }
}

impl<T, A> Port<T, A>
where
    A: Read,
    T: PortValue,
{
    /// # Safety
    /// A read to this port must not compromise memory safety.
    #[inline]
    pub unsafe fn read(&self) -> T {
        T::read(self.port)
    }
}

impl<T, A> Port<T, A>
where
    T: PortValue,
    A: Write,
{
    /// # Safety
    /// A write to this port must not compromise memory safety.
    #[inline]
    pub unsafe fn write(&self, value: T) {
        T::write(self.port, value);
    }
}

pub trait PortValue {
    /// # Safety
    /// A read to this port must not compromise memory safety.
    unsafe fn read(port: u16) -> Self;
    /// # Safety
    /// A write to this port must not compromise memory safety.
    unsafe fn write(port: u16, value: Self);
}

impl PortValue for u8 {
    #[inline]
    unsafe fn read(port: u16) -> Self {
        port_read_u8(port)
    }

    #[inline]
    unsafe fn write(port: u16, value: Self) {
        port_write_u8(port, value)
    }
}

impl PortValue for u16 {
    #[inline]
    unsafe fn read(port: u16) -> Self {
        port_read_u16(port)
    }

    #[inline]
    unsafe fn write(port: u16, value: Self) {
        port_write_u16(port, value)
    }
}

impl PortValue for u32 {
    #[inline]
    unsafe fn read(port: u16) -> Self {
        port_read_u32(port)
    }

    #[inline]
    unsafe fn write(port: u16, value: Self) {
        port_write_u32(port, value)
    }
}
