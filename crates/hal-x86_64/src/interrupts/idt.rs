use core::marker::PhantomData;

use hal_core::{self, volatile};
use x86_64::structures::idt::InterruptStackFrameValue;

use self::private::Sealed;
use crate::{addr::VirtAddr, intrin::lidt};

pub trait InterruptHandler {
    fn handle(stack_frame: &mut StackFrame);
}

pub trait ExceptionHandler {
    type Error;
    type Output;

    fn handle(stack_frame: &mut StackFrame, err: Self::Error) -> Self::Output;
}

#[repr(C, align(16))]
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
    pub page_fault: ExceptionEntry<PageFaultError, ()>,
    _reserved0: IdtEntry,
    pub x87_floating_point: ExceptionEntry<(), ()>,
    pub alignment_check: ExceptionEntry<u64, ()>,
    pub machine_check: ExceptionEntry<(), !>,
    pub simd_floating_point: ExceptionEntry<(), ()>,
    pub virtualization: ExceptionEntry<(), ()>,
    _reserved1: [IdtEntry; 8],
    pub vmm_communication_exception: ExceptionEntry<u64, ()>,
    pub security_exception: ExceptionEntry<u64, ()>,
    _reserved2: IdtEntry,
    pub interrupts: [InterruptEntry; N],
}

#[derive(Debug)]
pub struct PageFaultError {}

impl<const N: usize> InterruptTable<N> {
    pub const fn new() -> Self {
        InterruptTable {
            divide_error: ExceptionEntry::new(),
            debug: ExceptionEntry::new(),
            non_maskable_interrupt: ExceptionEntry::new(),
            breakpoint: ExceptionEntry::new(),
            overflow: ExceptionEntry::new(),
            bound_range_exceeded: ExceptionEntry::new(),
            invalid_opcode: ExceptionEntry::new(),
            device_not_available: ExceptionEntry::new(),
            double_fault: ExceptionEntry::new(),
            invalid_tss: ExceptionEntry::new(),
            segment_not_present: ExceptionEntry::new(),
            stack_segment_fault: ExceptionEntry::new(),
            general_protection_fault: ExceptionEntry::new(),
            page_fault: ExceptionEntry::new(),
            x87_floating_point: ExceptionEntry::new(),
            alignment_check: ExceptionEntry::new(),
            machine_check: ExceptionEntry::new(),
            simd_floating_point: ExceptionEntry::new(),
            virtualization: ExceptionEntry::new(),
            vmm_communication_exception: ExceptionEntry::new(),
            security_exception: ExceptionEntry::new(),
            interrupts: [InterruptEntry::new(); N],
            coprocessor_segment_overrun: ExceptionEntry::new(),
            _reserved0: IdtEntry::new(),
            _reserved1: [IdtEntry::new(); 8],
            _reserved2: IdtEntry::new(),
        }
    }

    pub fn load(&'static self) {
        unsafe {
            lidt(self, N + 32);
        }
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct ExceptionEntry<E, R>(IdtEntry, PhantomData<(E, R)>);

impl<E, R> ExceptionEntry<E, R> {
    const fn new() -> Self {
        Self(IdtEntry::new(), PhantomData)
    }

    pub fn set_handler<H>(&mut self)
    where
        H: ExceptionHandler<Error = E, Output = R>,
    {
        unsafe {
            let f = exception_trampoline::<H>;
            self.0.set_handler_address(f as usize)
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct InterruptEntry(IdtEntry);

impl InterruptEntry {
    const fn new() -> Self {
        Self(IdtEntry::new())
    }

    pub fn set_handler<H>(&mut self)
    where
        H: InterruptHandler,
    {
        unsafe {
            let f = interrupt_trampoline::<H>;
            self.0.set_handler_address(f as usize)
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct IdtEntry {
    pointer_low: u16,
    gdt_selector: u16,
    options: u16,
    pointer_middle: u16,
    pointer_high: u32,
    reserved: u32,
}

impl IdtEntry {
    pub const fn new() -> Self {
        Self {
            pointer_low: 0,
            gdt_selector: 0,
            options: 0b1110_0000_0000,
            pointer_middle: 0,
            pointer_high: 0,
            reserved: 0,
        }
    }

    pub unsafe fn set_handler_address(&mut self, addr: usize) {
        todo!()
    }
}

#[inline]
extern "x86-interrupt" fn interrupt_trampoline<H>(mut stack_frame: StackFrame)
where
    H: InterruptHandler,
{
    H::handle(&mut stack_frame);
}

#[inline]
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
#[derive(Debug)]
pub struct StackFrame {
    inner: InterruptStackFrameValue,
}

pub trait StackFrameExt: Sealed {
    fn instruction_pointer(&self) -> VirtAddr;
    fn code_segment(&self) -> u64;
    fn cpu_flags(&self) -> u64;
    fn stack_pointer(&self) -> VirtAddr;
    fn stack_segment(&self) -> u64;

    unsafe fn set_instruction_pointer(&mut self, instruction_pointer: VirtAddr);
    unsafe fn set_code_segment(&mut self, code_segment: u64);
    unsafe fn set_cpu_flags(&mut self, cpu_flags: u64);
    unsafe fn set_stack_pointer(&mut self, stack_pointer: VirtAddr);
    unsafe fn set_stack_segment(&mut self, stack_segment: u64);
}

impl Sealed for StackFrame {}

impl StackFrameExt for StackFrame {
    fn instruction_pointer(&self) -> VirtAddr {
        unsafe { VirtAddr::new_unchecked(self.inner.instruction_pointer.as_u64()) }
    }

    fn code_segment(&self) -> u64 {
        self.inner.code_segment
    }

    fn cpu_flags(&self) -> u64 {
        self.inner.cpu_flags
    }

    fn stack_pointer(&self) -> VirtAddr {
        unsafe { VirtAddr::new_unchecked(self.inner.stack_pointer.as_u64()) }
    }

    fn stack_segment(&self) -> u64 {
        self.inner.stack_segment
    }

    unsafe fn set_instruction_pointer(&mut self, instruction_pointer: VirtAddr) {
        let instruction_pointer =
            unsafe { x86_64::VirtAddr::new_unsafe(instruction_pointer.to_u64()) };
        volatile::write(&mut self.inner.instruction_pointer, instruction_pointer);
    }

    unsafe fn set_code_segment(&mut self, code_segment: u64) {
        volatile::write(&mut self.inner.code_segment, code_segment);
    }

    unsafe fn set_cpu_flags(&mut self, cpu_flags: u64) {
        volatile::write(&mut self.inner.cpu_flags, cpu_flags);
    }

    unsafe fn set_stack_pointer(&mut self, stack_pointer: VirtAddr) {
        let stack_pointer = unsafe { x86_64::VirtAddr::new_unsafe(stack_pointer.to_u64()) };
        volatile::write(&mut self.inner.stack_pointer, stack_pointer);
    }

    unsafe fn set_stack_segment(&mut self, stack_segment: u64) {
        volatile::write(&mut self.inner.stack_segment, stack_segment);
    }
}

mod private {
    pub trait Sealed {}
}
