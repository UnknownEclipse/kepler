use core::marker::PhantomData;

use self::private::Sealed;
use crate::access::{Read, ReadOnly, ReadWrite, Write, WriteOnly};

/// An x86 I/O port.
///
/// A `Port<T>` object can be though of as a `NonNull<T>`, except it refers to a
/// location in the I/O space. As such, it is cheaply const constructible and any number
/// of instances can refer to the same location. However as a result reads and writes
/// are `unsafe`.
///
/// # Usage
///
/// Enabling the [RTC](https://en.wikipedia.org/wiki/Real-time_clock):
/// ```
/// const RTC: Port<u8> = Port::new(0x70);
/// const CMOS: Port<u8> = Port::new(0x71);
///
/// unsafe { RTC.write(0x8a) };
/// unsafe { CMOS.write(0x20) };
/// ```
#[derive(Debug)]
pub struct Port<T, A = ReadWrite> {
    port: u16,
    _p: PhantomData<fn(T, A)>,
}

impl<T> Port<T> {
    pub const fn new(port: u16) -> Self {
        Self {
            port,
            _p: PhantomData,
        }
    }
}

impl<T> Port<T, ReadOnly> {
    pub const fn new_readonly(port: u16) -> Self {
        Self {
            port,
            _p: PhantomData,
        }
    }
}

impl<T> Port<T, WriteOnly> {
    pub const fn new_writeonly(port: u16) -> Self {
        Self {
            port,
            _p: PhantomData,
        }
    }
}

impl<T, A> Port<T, A>
where
    T: PortAccess,
    A: Read,
{
    /// # Safety
    /// 1. The port must be a valid I/O location
    /// 2. Reads from this port must not compromise memory safety.
    pub unsafe fn read(&self) -> T {
        T::read(self.port)
    }
}

impl<T, A> Port<T, A>
where
    T: PortAccess,
    A: Write,
{
    /// # Safety
    /// 1. The port must be a valid I/O location
    /// 2. Writes to this port must not compromise memory safety.
    pub unsafe fn write(&self, value: T) {
        T::write(self.port, value);
    }
}

pub trait PortAccess: Sealed {
    /// # Safety
    /// 1. The port must be a valid I/O location
    /// 2. Reads from this port must not compromise memory safety.
    unsafe fn read(port: u16) -> Self;
    /// # Safety
    /// 1. The port must be a valid I/O location
    /// 2. Writes to this port must not compromise memory safety.
    unsafe fn write(port: u16, value: Self);
}

mod private {
    pub trait Sealed {}
}
