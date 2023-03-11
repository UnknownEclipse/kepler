use hal::{
    interrupts::{self, enable, ExceptionHandler, InterruptTable, StackFrame},
    x86_64::interrupts::PageFaultError,
};
use log::{info, trace};
use spin::Lazy;

pub unsafe fn init() {
    debug_assert!(!interrupts::are_enabled());

    trace!("beginning initialization");
    let table = &*TABLE;
    trace!("loading interrupt table");
    table.load();
    trace!("finished loading interrupt table");

    enable();
    trace!("enabled interrupts");

    trace!("finished initialization");
}

static TABLE: Lazy<InterruptTable> = Lazy::new(build_table);

fn build_table() -> InterruptTable {
    trace!("building interrupt table");
    let mut table = InterruptTable::new();
    table.breakpoint.set_handler::<BreakpointHandler>();
    table.page_fault.set_handler::<PageFaultHandler>();
    table.double_fault.set_handler::<DoubleFaultHandler>();
    table
        .general_protection_fault
        .set_handler::<GeneralProtectionFaultHandler>();
    trace!("finished building interrupt table");
    table
}

#[derive(Debug)]
struct DoubleFaultHandler;

impl ExceptionHandler for DoubleFaultHandler {
    type Error = u64;
    type Output = !;

    fn handle(stack_frame: &mut StackFrame, error: Self::Error) -> Self::Output {
        panic!("double fault: {:#x}: {:#?}", error, stack_frame);
    }
}

#[derive(Debug)]
struct GeneralProtectionFaultHandler;

impl ExceptionHandler for GeneralProtectionFaultHandler {
    type Error = u64;
    type Output = ();

    fn handle(stack_frame: &mut StackFrame, error: Self::Error) -> Self::Output {
        panic!("general protection fault: {:#x}: {:#?}", error, stack_frame);
    }
}

#[derive(Debug)]
struct PageFaultHandler;

impl ExceptionHandler for PageFaultHandler {
    type Error = PageFaultError;
    type Output = ();

    fn handle(stack_frame: &mut StackFrame, error: Self::Error) -> Self::Output {
        if error.is_protection_violation() {
            panic!("page fault: {:#x}: {:#?}", error, stack_frame);
        } else if error.is_user() {
            // Check if address is within user heap region to determine if it's safe to
            // map.
            todo!("userspace lazy page mapping")
        }
        todo!("lazy page mapping");
    }
}

#[derive(Debug)]
struct BreakpointHandler;

impl ExceptionHandler for BreakpointHandler {
    type Error = ();
    type Output = ();

    fn handle(_stack_frame: &mut StackFrame, _error: ()) {
        info!("breakpoint");
    }
}
