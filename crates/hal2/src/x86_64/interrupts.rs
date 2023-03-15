use core::num::NonZeroU8;

use vm_types::VirtAddr;

use self::private::Sealed;
use super::{reg::cs, tss::IstIndex};
pub use crate::x86_common::interrupts::*;
use crate::{read_volatile, write_volatile};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct StackFrame {
    rip: VirtAddr,
    cs: usize,
    rflags: usize,
    rsp: VirtAddr,
}

impl Sealed for StackFrame {}

impl X86_64StackFrameExt for StackFrame {
    #[inline]
    fn rip(&self) -> VirtAddr {
        read_volatile(&self.rip)
    }

    #[inline]
    fn cs(&self) -> usize {
        read_volatile(&self.cs)
    }

    #[inline]
    fn rflags(&self) -> usize {
        read_volatile(&self.rflags)
    }

    #[inline]
    fn rsp(&self) -> VirtAddr {
        read_volatile(&self.rsp)
    }

    #[inline]
    unsafe fn set_rip(&mut self, rip: VirtAddr) {
        write_volatile(&mut self.rip, rip);
    }

    #[inline]
    unsafe fn set_cs(&mut self, cs: usize) {
        write_volatile(&mut self.cs, cs);
    }

    #[inline]
    unsafe fn set_rflags(&mut self, rflags: usize) {
        write_volatile(&mut self.rflags, rflags);
    }

    #[inline]
    unsafe fn set_rsp(&mut self, rsp: VirtAddr) {
        write_volatile(&mut self.rsp, rsp);
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct InterruptGate {
    offset_low: u16,
    pub segment_selector: u16,
    pub ist: Option<NonZeroU8>,
    pub options: GateOptions,
    offset_mid: u16,
    offset_high: u32,
    _reserved: u32,
}

impl InterruptGate {
    pub const fn new() -> Self {
        Self {
            offset_low: 0,
            segment_selector: 0,
            ist: None,
            options: GateOptions::new(GateKind::Interrupt),
            offset_mid: 0,
            offset_high: 0,
            _reserved: 0,
        }
    }

    pub fn set_handler_addr(&mut self, addr: VirtAddr) -> &mut Self {
        let addr = addr.as_usize();
        self.offset_low = addr as u16;
        self.offset_mid = addr.wrapping_shr(16) as u16;
        self.offset_high = addr.wrapping_shr(32) as u32;
        self.segment_selector = unsafe { cs::read() };
        self.options.set_present(true);
        self
    }

    pub fn set_ist_index(&mut self, index: IstIndex) {
        self.ist = Some(index.0);
    }
}

pub trait X86_64StackFrameExt: Sealed {
    fn rip(&self) -> VirtAddr;
    fn cs(&self) -> usize;
    fn rflags(&self) -> usize;
    fn rsp(&self) -> VirtAddr;
    unsafe fn set_rip(&mut self, rip: VirtAddr);
    unsafe fn set_cs(&mut self, cs: usize);
    unsafe fn set_rflags(&mut self, rflags: usize);
    unsafe fn set_rsp(&mut self, rsp: VirtAddr);
}

pub trait X86IdtEntryExt {
    fn set_ist_index(&mut self, index: IstIndex);
}

mod private {
    pub trait Sealed {}
}
