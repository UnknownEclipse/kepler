use core::{marker::PhantomData, num::NonZeroU16};

use vm_types::VirtAddr;

use super::{
    instr::{cli, hlt, lidt, sti, sti_hlt},
    reg::flags,
};
use crate::{
    interrupts::{ExceptionHandler, InterruptHandler, StackFrame},
    x86_64::interrupts::InterruptGate,
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

#[derive(Debug)]
pub struct InterruptTable<const N: usize = 224> {
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
    pub page_fault: ExceptionEntry<u64, ()>,
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
    interrupts: [InterruptEntry; N],
}

impl<const N: usize> InterruptTable<N> {
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
            interrupts: [InterruptEntry::empty(); N],
            coprocessor_segment_overrun: ExceptionEntry::empty(),
            _reserved0: InterruptGate::new(),
            _reserved1: [InterruptGate::new(); 8],
            _reserved2: InterruptGate::new(),
        }
    }

    pub fn load(&'static self) {
        unsafe { lidt(self as *const Self as *const u8, N + 32) };
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

    fn set_raw_handler(&mut self, handler: extern "x86-interrupt" fn(StackFrame)) {
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

    fn set_raw_handler(&mut self, handler: extern "x86-interrupt" fn(StackFrame, E) -> R) {
        let addr = handler as usize;
        self.gate.set_handler_addr(VirtAddr::from_usize(addr));
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
pub(crate) struct GateOptions(u16);

impl GateOptions {
    pub const fn new(kind: GateKind) -> Self {
        Self(kind.bits() << 8)
    }

    pub fn set_present(&mut self, present: bool) -> &mut Self {
        self.0 = bitfrob::u16_with_bit(15, self.0, present);
        self
    }

    pub fn set_gate_kind(&mut self, kind: GateKind) -> &mut Self {
        self.0 = bitfrob::u16_with_value(8, 11, self.0, kind.bits());
        self
    }

    pub fn set_interrupt_stack_table_index(&mut self, index: Option<NonZeroU16>) -> &mut Self {
        let index = index.map(|v| v.get()).unwrap_or(0);
        self.0 = bitfrob::u16_with_value(0, 2, self.0, index);
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum GateKind {
    Interrupt,
    Trap,
}

impl GateKind {
    const fn bits(&self) -> u16 {
        match self {
            GateKind::Interrupt => 0xe,
            GateKind::Trap => 0xf,
        }
    }
}
