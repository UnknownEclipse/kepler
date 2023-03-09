#![no_std]
#![no_main]
#![feature(
    abi_x86_interrupt,
    allocator_api,
    atomic_mut_ptr,
    never_type,
    nonnull_slice_from_raw_parts,
    step_trait,
    sync_unsafe_cell
)]

extern crate alloc;

use core::panic::PanicInfo;

use hal::interrupts;
use log::{error, info, set_logger, set_max_level, trace, LevelFilter};
use stdio::StdoutLogger;

mod interrupt_table;
mod memory;
mod stdio;
mod task;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    set_logger(&StdoutLogger).expect("no other logger is set");
    set_max_level(LevelFilter::Trace);

    info!("kepler v0.1.0");
    info!("beginning initialization");

    unsafe {
        // These submodules all use Lazy/Once internally, however we choose to
        // explicitly initialize them here to make the ordering consistent and prevent
        // any kind of associated bugs.

        interrupt_table::init();
        memory::init();
        task::enter(0);
    }

    trace!("finished initialization");

    loop {
        unsafe { interrupts::wait() };
    }
}

#[track_caller]
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    error!("{}", panic_info);
    loop {
        unsafe { interrupts::wait() };
    }
}
