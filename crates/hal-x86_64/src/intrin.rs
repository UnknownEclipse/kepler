//! Intrinsic definitions for assembly operations.

#![allow(clippy::missing_safety_doc)]

use core::arch::{
    asm,
    x86_64::{_rdrand16_step, _rdrand32_step, _rdrand64_step},
};

use x86_64::{
    instructions::{
        hlt,
        interrupts::{self, int3},
        port::Port,
    },
    registers::{model_specific::Msr, rflags},
};

/// # Safety
/// This is an intrinsic
#[inline]
pub unsafe fn halt() {
    hlt();
}

/// # Safety
/// This is an intrinsic
#[inline]
pub unsafe fn enable_interrupts() {
    interrupts::enable();
}

/// # Safety
/// This is an intrinsic
#[inline]
pub unsafe fn enable_interrupts_and_halt() {
    interrupts::enable_and_hlt();
}

/// # Safety
/// This is an intrinsic
#[inline]
pub unsafe fn disable_interrupts() {
    interrupts::disable();
}

/// # Safety
/// This is an intrinsic
#[inline]
pub unsafe fn interrupts_are_enabled() -> bool {
    interrupts::are_enabled()
}

#[inline]
pub unsafe fn port_write_u8(port: u16, value: u8) {
    Port::new(port).write(value);
}

#[inline]
pub unsafe fn port_write_u16(port: u16, value: u16) {
    Port::new(port).write(value);
}

#[inline]
pub unsafe fn port_write_u32(port: u16, value: u32) {
    Port::new(port).write(value);
}

#[inline]
pub unsafe fn port_read_u8(port: u16) -> u8 {
    Port::new(port).read()
}

#[inline]
pub unsafe fn port_read_u16(port: u16) -> u16 {
    Port::new(port).read()
}

#[inline]
pub unsafe fn port_read_u32(port: u16) -> u32 {
    Port::new(port).read()
}

#[inline]
pub unsafe fn read_rflags() -> u64 {
    rflags::read_raw()
}

#[inline]
pub unsafe fn write_rflags(value: u64) {
    rflags::write_raw(value);
}

#[inline]
pub unsafe fn breakpoint() {
    int3();
}

#[inline]
pub unsafe fn random_u16() -> Option<u16> {
    let mut value = 0;
    if 0 == _rdrand16_step(&mut value) {
        None
    } else {
        Some(value)
    }
}

#[inline]
pub unsafe fn random_u32() -> Option<u32> {
    let mut value = 0;
    if 0 == _rdrand32_step(&mut value) {
        None
    } else {
        Some(value)
    }
}

#[inline]
pub unsafe fn random_u64() -> Option<u64> {
    let mut value = 0;
    if 0 == _rdrand64_step(&mut value) {
        None
    } else {
        Some(value)
    }
}

#[repr(C, packed(2))]
#[derive(Debug, Clone, Copy)]
struct DescriptorTablePtr {
    pub limit: u16,
    pub base: *const u8,
}

#[inline]
pub unsafe fn lidt<T>(ptr: *const T, entries: usize) {
    let idt = &DescriptorTablePtr {
        limit: (entries * 16) as u16 - 1,
        base: ptr.cast(),
    };

    asm!("lidt [{}]", in(reg) idt, options(readonly, nostack, preserves_flags));
}

#[inline]
pub unsafe fn lgdt<T>(ptr: *const T, entries: usize) {
    let idt = &DescriptorTablePtr {
        limit: (entries * 8) as u16 - 1,
        base: ptr.cast(),
    };

    asm!("lgdt [{}]", in(reg) idt, options(readonly, nostack, preserves_flags));
}

#[inline]
pub unsafe fn hw_thread_id() -> u32 {
    const IA32_TSC_AUX: u32 = 0xc0000103;
    Msr::new(IA32_TSC_AUX).read() as u32
}
