use hal::interrupts::{self, enable, ExceptionHandler, InterruptTable};
use log::{info, trace};
use spin::{Lazy, Once};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

pub unsafe fn init() {
    interrupts::without(|_| {
        LOAD.call_once(|| {
            trace!("beginning initialization");
            let idt = &*IDT;
            trace!("loading interrupt table");
            idt.load();
            trace!("finished loading interrupt table");
            trace!("finished initialization");
        });
    });

    enable();
}

static LOAD: Once = Once::new();
static TABLE: Lazy<InterruptTable> = Lazy::new(build_table);
static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(build_idt);

fn build_idt() -> InterruptDescriptorTable {
    let mut idt = InterruptDescriptorTable::new();
    idt.double_fault.set_handler_fn(double_fault_handler);
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);
    idt
}

fn build_table() -> InterruptTable {
    trace!("building interrupt table");
    let mut table = InterruptTable::new();
    table.breakpoint.set_handler::<BreakpointHandler>();
    table.double_fault.set_handler::<DoubleFaultHandler>();

    // table.page_fault.set_handler::<PageFaultHandler>();
    trace!("finished building interrupt table");
    table
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, error: u64) -> ! {
    panic!("double fault: {:#x}: {:#?}", error, stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error: PageFaultErrorCode,
) {
    panic!("page fault: {:#x}: {:#?}", error, stack_frame);
}
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    info!("breakpoint");
}

#[derive(Debug)]
struct DoubleFaultHandler;

impl ExceptionHandler for DoubleFaultHandler {
    type Error = u64;
    type Output = !;

    fn handle(stack_frame: &mut interrupts::StackFrame, error: Self::Error) -> Self::Output {
        panic!("double fault: {:#x}: {:#?}", error, stack_frame);
    }
}

#[derive(Debug)]
struct BreakpointHandler;

impl ExceptionHandler for BreakpointHandler {
    type Error = ();
    type Output = ();

    fn handle(_stack_frame: &mut interrupts::StackFrame, _error: ()) {
        info!("breakpoint");
    }
}
