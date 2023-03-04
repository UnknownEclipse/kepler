use hal::{
    interrupts::{ExceptionHandler, InterruptHandler, InterruptTable, PageFaultError, StackFrame},
    intrin::enable_interrupts,
};
use pic8259::ChainedPics;
use spin::{mutex::SpinMutex, Lazy};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use crate::stdout::serial;

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(build_idt);
static CHAINED_PICS: SpinMutex<ChainedPics> = unsafe { SpinMutex::new(ChainedPics::new(32, 40)) };

static TABLE: Lazy<InterruptTable> = Lazy::new(build_table);

pub fn init() {
    IDT.load();

    unsafe { enable_interrupts() };

    let mut pics = CHAINED_PICS.lock();
    unsafe {
        pics.initialize();
        pics.write_masks(!1, u8::MAX);
    };
}

fn build_table() -> InterruptTable {
    let mut table = InterruptTable::new();
    table.double_fault.set_handler::<DoubleFaultHandler>();
    table.page_fault.set_handler::<PageFaultHandler>();
    table.interrupts[0].set_handler::<TimerHandler>();
    table.breakpoint.set_handler::<BreakpointHandler>();
    table
}

fn build_idt() -> InterruptDescriptorTable {
    let mut idt = InterruptDescriptorTable::new();

    idt.double_fault.set_handler_fn(double_fault_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);

    idt[32].set_handler_fn(timer_handler);
    idt
}

#[derive(Debug)]
struct PageFaultHandler;

impl ExceptionHandler for PageFaultHandler {
    type Error = PageFaultError;
    type Output = ();

    fn handle(_frame: &mut StackFrame, _error: Self::Error) -> Self::Output {
        panic!("EXCEPTION: PAGE FAULT");
    }
}

#[derive(Debug)]
struct DoubleFaultHandler;

impl ExceptionHandler for DoubleFaultHandler {
    type Error = u64;
    type Output = !;

    fn handle(_frame: &mut StackFrame, _error: Self::Error) -> Self::Output {
        panic!("EXCEPTION: DOUBLE FAULT");
    }
}

#[derive(Debug)]
struct TimerHandler;

impl InterruptHandler for TimerHandler {
    fn handle(_frame: &mut StackFrame) {
        serial::print!(".");
        unsafe {
            CHAINED_PICS.lock().notify_end_of_interrupt(32);
        }
    }
}

#[derive(Debug)]
struct BreakpointHandler;

impl ExceptionHandler for BreakpointHandler {
    type Error = ();
    type Output = ();

    fn handle(_stack_frame: &mut StackFrame, _err: Self::Error) -> Self::Output {
        log::info!("BREAKPOINT");
    }
}

extern "x86-interrupt" fn timer_handler(frame: InterruptStackFrame) {
    serial::print!(".");
    unsafe {
        CHAINED_PICS.lock().notify_end_of_interrupt(32);
    }
}

extern "x86-interrupt" fn page_fault_handler(
    frame: InterruptStackFrame,
    _error: PageFaultErrorCode,
) {
    panic!("EXCEPTION: PAGE FAULT");
}

extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, _error: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT");
}
