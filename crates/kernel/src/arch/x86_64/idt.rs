use hal::vm_types::{PhysAddr, VirtAddr};
use log::{error, trace};
use spin::{mutex::SpinMutex, Lazy, Once};
use x2apic::lapic::{xapic_base, IpiAllShorthand, LocalApic, LocalApicBuilder};
use x86_64::{
    registers::control::Cr2,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode},
};

use super::interrupts;
use crate::{
    arch::IpiTarget,
    memory::{self, map_physical_addr},
};

pub static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(build_idt);

pub unsafe fn init() {
    IDT.load();

    let apic_physical_address = unsafe { PhysAddr::from_usize(xapic_base() as usize) };
    let apic_virtual_address = unsafe { map_physical_addr(apic_physical_address) };

    let mut apic = LOCAL_APIC
        .try_call_once(|| {
            LocalApicBuilder::new()
                .error_vector(InterruptVector::LocalApicError as usize)
                .spurious_vector(InterruptVector::SpuriousInterrupt as usize)
                .timer_vector(InterruptVector::Timer as usize)
                .set_xapic_base(apic_virtual_address.as_usize() as u64)
                .build()
                .map(SpinMutex::new)
        })
        .expect("local apic init error")
        .lock();

    unsafe {
        apic.enable();
    }

    trace!("apic id := {}", apic.id());

    interrupts::enable();
}

pub unsafe fn send_ipi(target: IpiTarget) {
    let mut apic = LOCAL_APIC.get().unwrap().lock();
    let vector = InterruptVector::Ipi as u8;

    match target {
        IpiTarget::Others => unsafe {
            apic.send_ipi_all(vector, IpiAllShorthand::AllExcludingSelf)
        },
        IpiTarget::Single(cpu) => {
            unsafe { apic.send_ipi(vector, cpu.0) };
        }
    }
}

pub static LOCAL_APIC: Once<SpinMutex<LocalApic>> = Once::new();

#[repr(u8)]
#[derive(Debug)]
enum InterruptVector {
    Timer = 32,
    LocalApicError = 40,
    SpuriousInterrupt = 41,
    Ipi = 42,
    Syscall = 0x80,
}

fn build_idt() -> InterruptDescriptorTable {
    let mut idt = InterruptDescriptorTable::new();

    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt[32].set_handler_fn(timer_handler);

    unsafe {
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault_handler)
            .set_stack_index(0);

        idt.double_fault
            .set_handler_fn(double_fault_handler)
            .set_stack_index(0);

        idt.page_fault
            .set_handler_fn(page_fault_handler)
            .set_stack_index(0);
    };

    idt
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error: PageFaultErrorCode,
) {
    if error.contains(PageFaultErrorCode::PROTECTION_VIOLATION) {
        panic!(
            "page fault at {:p}: {:?}: {:#?}",
            Cr2::read(),
            error,
            stack_frame
        );
    } else if error.contains(PageFaultErrorCode::USER_MODE) {
        // Check if address is within user's allocated heap region to determine if it's
        // safe to map.
        todo!("userspace lazy page mapping")
    }

    trace!("page fault, handling as lazily mapped page");
    let addr = Cr2::read_raw();
    let addr = VirtAddr::from_usize(addr as usize);
    unsafe { memory::handle_page_fault(addr).expect("page fault failure") };
}

extern "x86-interrupt" fn general_protection_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
    panic!("general protection fault");
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!("double fault: {:#?}: {:#?}", stack_frame, error_code);
}

extern "x86-interrupt" fn breakpoint_handler(_stack_frame: InterruptStackFrame) {
    trace!("breakpoint");
}

extern "x86-interrupt" fn timer_handler(_stack_frame: InterruptStackFrame) {
    trace!("timer");

    unsafe {
        if let Some(apic) = LOCAL_APIC.get() {
            apic.lock().end_of_interrupt();
        }
    }
}

extern "x86-interrupt" fn spurious_interrupt_handler(_stack_frame: InterruptStackFrame) {
    trace!("spurious interrupt");

    unsafe {
        let mut apic = LOCAL_APIC.get().unwrap().lock();
        apic.end_of_interrupt();
    }
}

extern "x86-interrupt" fn apic_error_handler(_stack_frame: InterruptStackFrame) {
    let mut lapic = LOCAL_APIC.get().unwrap().lock();

    error!("apic error: {:?}", unsafe { lapic.error_flags() });

    unsafe {
        lapic.end_of_interrupt();
    }
}
