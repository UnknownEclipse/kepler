#![no_std]
#![feature(abi_x86_interrupt, asm_const, naked_functions, never_type)]

use core::ptr;

pub use vm_types;

#[cfg(target_arch = "x86")]
use crate::x86 as imp;
#[cfg(target_arch = "x86_64")]
use crate::x86_64 as imp;

#[cfg(target_arch = "x86")]
pub mod x86;
#[cfg(target_arch = "x86_64")]
pub mod x86_64;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86_common;

pub mod access;
pub mod interrupts;
pub mod paging;
pub mod task;

fn read_volatile<T>(value: &T) -> T
where
    T: Copy,
{
    unsafe { ptr::read_volatile(value) }
}

fn write_volatile<T>(dst: &mut T, value: T)
where
    T: Copy,
{
    unsafe { ptr::write_volatile(dst, value) };
}
