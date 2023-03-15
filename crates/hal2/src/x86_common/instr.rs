#![allow(clippy::missing_safety_doc)]

use core::arch::{
    asm,
    x86_64::{
        _rdrand16_step, _rdrand32_step, _rdrand64_step, _rdseed16_step, _rdseed32_step,
        _rdseed64_step,
    },
};

use vm_types::VirtAddr;

use super::interrupts::IdtPtr;
use crate::x86_64::gdt::GdtPtr;

#[inline]
pub unsafe fn out8(port: u16, value: u8) {
    asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
}

#[inline]
pub unsafe fn out16(port: u16, value: u16) {
    asm!("out dx, ax", in("dx") port, in("ax") value, options(nomem, nostack, preserves_flags));
}

#[inline]
pub unsafe fn out32(port: u16, value: u16) {
    asm!("out dx, eax", in("dx") port, in("eax") value, options(nomem, nostack, preserves_flags));
}

#[inline]
pub unsafe fn in8(port: u16) -> u8 {
    let mut value: u8;
    asm!("in al, dx", out("al") value, in("dx") port, options(nomem, nostack, preserves_flags));
    value
}

#[inline]
pub unsafe fn in16(port: u16) -> u16 {
    let mut value: u16;
    asm!("in ax, dx", out("ax") value, in("dx") port, options(nomem, nostack, preserves_flags));
    value
}

#[inline]
pub unsafe fn in32(port: u16) -> u32 {
    let mut value: u32;
    asm!("in eax, dx", out("eax") value, in("dx") port, options(nomem, nostack, preserves_flags));
    value
}

#[inline]
pub unsafe fn rdmsr(msr: u32) -> u64 {
    let mut high: u32;
    let mut low: u32;
    asm!("rdmsr", in("ecx") msr, out("edx") high, out("eax") low, options(nomem, nostack, preserves_flags));
    ((high as u64) << 32) | (low as u64)
}

#[inline]
pub unsafe fn wrmsr(msr: u32, value: u64) {
    let high = value.wrapping_shr(32) as u32;
    let low = value as u32;
    asm!("wrmsr", in("ecx") msr, in("edx") high, in("eax") low, options(nomem, nostack, preserves_flags));
}

#[inline]
pub unsafe fn sti() {
    asm!("sti", options(nomem, nostack, preserves_flags));
}

#[inline]
pub unsafe fn cli() {
    asm!("cli", options(nomem, nostack, preserves_flags));
}

#[inline]
pub unsafe fn sti_hlt() {
    asm!("sti; hlt", options(nomem, nostack, preserves_flags));
}

#[inline]
pub unsafe fn hlt() {
    asm!("hlt", options(nomem, nostack, preserves_flags));
}

#[inline]
pub unsafe fn pause() {
    asm!("pause", options(nomem, nostack, preserves_flags));
}

#[inline]
pub unsafe fn rdpid() -> usize {
    let mut value: usize;
    asm!("rdpid {}", out(reg) value, options(nomem, nostack, preserves_flags));
    value
}

#[inline]
pub unsafe fn rdtsc() -> u64 {
    let high: u32;
    let low: u32;
    asm!("rdtsc", out("edx") high, out("eax") low, options(nomem, nostack, preserves_flags));
    pack_u64(high, low)
}

#[inline]
pub unsafe fn rdtscp() -> (u64, u32) {
    let high: u32;
    let low: u32;
    let aux: u32;
    asm!("rdtscp", out("edx") high, out("eax") low, out("ecx") aux, options(nomem, nostack, preserves_flags));
    (pack_u64(high, low), aux)
}

#[inline]
pub unsafe fn rdrand16() -> Option<u16> {
    let mut value = 0;
    if 1 == _rdrand16_step(&mut value) {
        Some(value)
    } else {
        None
    }
}

#[inline]
pub unsafe fn rdrand32() -> Option<u32> {
    let mut value = 0;
    if 1 == _rdrand32_step(&mut value) {
        Some(value)
    } else {
        None
    }
}

#[inline]
pub unsafe fn rdrand64() -> Option<u64> {
    let mut value = 0;
    if 1 == _rdrand64_step(&mut value) {
        Some(value)
    } else {
        None
    }
}

#[inline]
pub unsafe fn rdseed16() -> Option<u16> {
    let mut value = 0;
    if 1 == _rdseed16_step(&mut value) {
        Some(value)
    } else {
        None
    }
}

#[inline]
pub unsafe fn rdseed32() -> Option<u32> {
    let mut value = 0;
    if 1 == _rdseed32_step(&mut value) {
        Some(value)
    } else {
        None
    }
}

#[inline]
pub unsafe fn rdseed64() -> Option<u64> {
    let mut value = 0;
    if 1 == _rdseed64_step(&mut value) {
        Some(value)
    } else {
        None
    }
}

#[inline]
pub unsafe fn lidt(ptr: *const IdtPtr) {
    asm!("lidt [{}]", in(reg) ptr, options(readonly, nostack, preserves_flags));
}

#[inline]
pub unsafe fn lgdt(ptr: *const GdtPtr) {
    asm!("lgdt [{}]", in(reg) ptr, options(nomem, nostack, preserves_flags));
}

#[inline]
pub unsafe fn int3() {
    asm!("int3", options(nomem, nostack));
}

#[inline]
pub unsafe fn int<const N: u32>() {
    asm!("int {}", const N, options(nomem, nostack));
}

#[inline]
pub unsafe fn invlpg(addr: *const ()) {
    asm!("invlpg [{}]", in(reg) addr, options(nomem, nostack, preserves_flags));
}

#[inline]
pub unsafe fn ltr(sel: u16) {
    asm!("ltr {0:x}", in(reg) sel, options(nostack, preserves_flags));
}

#[repr(C, packed(2))]
#[derive(Debug, Clone, Copy)]
struct DescTablePtr {
    pub limit: u16,
    pub base: VirtAddr,
}

#[inline]
fn pack_u64(high: u32, low: u32) -> u64 {
    ((high as u64) << 32) | (low as u64)
}
