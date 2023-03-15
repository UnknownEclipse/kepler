use core::{cell::SyncUnsafeCell, marker::PhantomData, mem};

use bitflags::bitflags;
use vm_types::VirtAddr;

use super::{
    instr::{cli, hlt, lidt, sti, sti_hlt},
    reg::flags,
};
use crate::{
    interrupts::{ExceptionHandler, InterruptHandler, StackFrame},
    x86_64::interrupts::{InterruptGate, X86IdtEntryExt},
};

#[inline]
pub(crate) unsafe fn enable() {
    sti()
}

#[inline]
pub(crate) unsafe fn enable_and_wait() {
    sti_hlt();
}

#[inline]
pub(crate) unsafe fn disable() {
    cli()
}

#[inline]
pub(crate) unsafe fn are_enabled() -> bool {
    flags::read() & (1 << 9) != 0
}

#[inline]
pub(crate) unsafe fn wait() {
    hlt()
}

#[repr(C, align(16))]
#[derive(Debug)]
pub struct InterruptTable {
    pub divide_error: ExceptionEntry<(), ()>,
    pub debug: ExceptionEntry<(), ()>,
    pub non_maskable_interrupt: ExceptionEntry<(), ()>,
    pub breakpoint: ExceptionEntry<(), ()>,
    pub overflow: ExceptionEntry<(), ()>,
    pub bound_range_exceeded: ExceptionEntry<(), ()>,
    pub invalid_opcode: ExceptionEntry<(), ()>,
    pub device_not_available: ExceptionEntry<(), ()>,
    pub double_fault: ExceptionEntry<u64, !>,
    coprocessor_segment_overrun: ExceptionEntry<(), ()>,
    pub invalid_tss: ExceptionEntry<u64, ()>,
    pub segment_not_present: ExceptionEntry<u64, ()>,
    pub stack_segment_fault: ExceptionEntry<u64, ()>,
    pub general_protection_fault: ExceptionEntry<u64, ()>,
    pub page_fault: ExceptionEntry<PageFaultError, ()>,
    _reserved0: InterruptGate,
    pub x87_floating_point: ExceptionEntry<(), ()>,
    pub alignment_check: ExceptionEntry<u64, ()>,
    pub machine_check: ExceptionEntry<(), !>,
    pub simd_floating_point: ExceptionEntry<(), ()>,
    pub virtualization: ExceptionEntry<(), ()>,
    _reserved1: [InterruptGate; 8],
    pub vmm_communication_exception: ExceptionEntry<u64, ()>,
    pub security_exception: ExceptionEntry<u64, ()>,
    _reserved2: InterruptGate,
    interrupts: [InterruptEntry; 224],
    ptr: SyncUnsafeCell<IdtPtr>,
}

#[repr(C, packed(2))]
#[derive(Debug, Clone, Copy, Default)]
pub struct IdtPtr {
    limit: u16,
    base: VirtAddr,
}

impl InterruptTable {
    pub const fn new() -> Self {
        InterruptTable {
            divide_error: ExceptionEntry::empty(),
            debug: ExceptionEntry::empty(),
            non_maskable_interrupt: ExceptionEntry::empty(),
            breakpoint: ExceptionEntry::empty(),
            overflow: ExceptionEntry::empty(),
            bound_range_exceeded: ExceptionEntry::empty(),
            invalid_opcode: ExceptionEntry::empty(),
            device_not_available: ExceptionEntry::empty(),
            double_fault: ExceptionEntry::empty(),
            invalid_tss: ExceptionEntry::empty(),
            segment_not_present: ExceptionEntry::empty(),
            stack_segment_fault: ExceptionEntry::empty(),
            general_protection_fault: ExceptionEntry::empty(),
            page_fault: ExceptionEntry::empty(),
            x87_floating_point: ExceptionEntry::empty(),
            alignment_check: ExceptionEntry::empty(),
            machine_check: ExceptionEntry::empty(),
            simd_floating_point: ExceptionEntry::empty(),
            virtualization: ExceptionEntry::empty(),
            vmm_communication_exception: ExceptionEntry::empty(),
            security_exception: ExceptionEntry::empty(),
            coprocessor_segment_overrun: ExceptionEntry::empty(),
            _reserved0: InterruptGate::new(),
            _reserved1: [InterruptGate::new(); 8],
            _reserved2: InterruptGate::new(),
            interrupts: [InterruptEntry::empty(); 256 - 32],
            ptr: SyncUnsafeCell::new(IdtPtr {
                limit: 0,
                base: VirtAddr::zero(),
            }),
        }
    }

    pub fn load(&'static self) {
        let slot = self.ptr.get();
        unsafe {
            slot.write(IdtPtr {
                limit: mem::size_of::<Self>() as u16 - 11,
                base: VirtAddr::from_ptr(self),
            })
        };
        unsafe { lidt(slot) };
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct InterruptEntry {
    gate: InterruptGate,
}

impl InterruptEntry {
    pub const fn empty() -> Self {
        InterruptEntry {
            gate: InterruptGate::new(),
        }
    }

    pub fn set_handler<H>(&mut self)
    where
        H: InterruptHandler,
    {
        self.set_raw_handler(interrupt_trampoline::<H>);
    }

    pub fn set_raw_handler(&mut self, handler: extern "x86-interrupt" fn(StackFrame)) {
        let addr = handler as usize;
        self.gate.set_handler_addr(VirtAddr::from_usize(addr));
    }
}

extern "x86-interrupt" fn interrupt_trampoline<H>(mut stack_frame: StackFrame)
where
    H: InterruptHandler,
{
    H::handle(&mut stack_frame);
}

bitflags! {
    pub struct PageFaultError: u64 {
        const PROTECTION_VIOLATION = 1 << 0;
        const WRITE = 1 << 1;
        const USER = 1 << 2;
        const RESERVED_WRITE = 1 << 3;
        const INSTRUCTION_FETCH = 1 << 4;
        const PROTECTION_KEY = 1 << 5;
        const SHADOW_STACK = 1 << 6;
        const SGX = 1 << 7;
    }
}

impl PageFaultError {
    pub fn is_protection_violation(&self) -> bool {
        self.contains(PageFaultError::PROTECTION_VIOLATION)
    }

    pub fn is_user(&self) -> bool {
        self.contains(PageFaultError::USER)
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct ExceptionEntry<E, R> {
    gate: InterruptGate,
    _p: PhantomData<fn(E, R)>,
}

impl<E, R> ExceptionEntry<E, R> {
    pub const fn empty() -> Self {
        ExceptionEntry {
            gate: InterruptGate::new(),
            _p: PhantomData,
        }
    }

    pub fn set_handler<H>(&mut self)
    where
        H: ExceptionHandler<Output = R, Error = E>,
    {
        self.set_raw_handler(exception_trampoline::<H>);
    }

    pub fn set_raw_handler(&mut self, handler: extern "x86-interrupt" fn(StackFrame, E) -> R) {
        let addr = handler as usize;
        self.gate.set_handler_addr(VirtAddr::from_usize(addr));
    }
}

impl<E, R> X86IdtEntryExt for ExceptionEntry<E, R> {
    fn set_ist_index(&mut self, index: crate::x86_64::tss::IstIndex) {
        self.gate.set_ist_index(index);
    }
}

extern "x86-interrupt" fn exception_trampoline<H>(
    mut stack_frame: StackFrame,
    error: H::Error,
) -> H::Output
where
    H: ExceptionHandler,
{
    H::handle(&mut stack_frame, error)
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct GateOptions(u8);

impl GateOptions {
    pub const fn new(kind: GateKind) -> Self {
        Self(kind.bits())
    }

    pub fn set_present(&mut self, present: bool) -> &mut Self {
        self.0 = bitfrob::u8_with_bit(7, self.0, present);
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum GateKind {
    Interrupt,
    Trap,
}

impl GateKind {
    const fn bits(&self) -> u8 {
        match self {
            GateKind::Interrupt => 0xe,
            GateKind::Trap => 0xf,
        }
    }
}
