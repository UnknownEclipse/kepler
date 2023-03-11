#![no_std]
#![no_main]
#![feature(
    abi_x86_interrupt,
    allocator_api,
    atomic_mut_ptr,
    custom_test_frameworks,
    error_in_core,
    never_type,
    nonnull_slice_from_raw_parts,
    step_trait,
    sync_unsafe_cell,
    vec_push_within_capacity
)]

extern crate alloc;

use core::panic::PanicInfo;

use error::KernResult;
use hal::{interrupts, task::hw_thread_id};
use limine::{LimineSmpInfo, LimineSmpRequest};
use log::{error, info, set_logger, set_max_level, trace, LevelFilter};
use stdio::StdoutLogger;

use crate::task::{spawn, yield_now};

mod error;
mod interrupt_table;
mod memory;
mod soul_local;
mod stdio;
mod sync;
mod syscall;
mod task;
mod x86;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    kernel_main().expect("kernel error occurred");

    loop {
        unsafe { interrupts::wait() };
    }
}

static SMP_REQUEST: LimineSmpRequest = LimineSmpRequest::new(0);

fn kernel_main() -> KernResult<()> {
    set_logger(&StdoutLogger).expect("no other logger is set");
    set_max_level(LevelFilter::Trace);

    info!("kepler v0.1.0");
    info!("beginning initialization");

    unsafe {
        // These submodules all use Lazy/Once internally, however we choose to
        // explicitly initialize them here to make the ordering consistent and prevent
        // any kind of associated bugs.

        interrupt_table::init();
        memory::init()?;
        hal::task::init_hw_thread(0);
    }

    trace!("finished initialization");

    task::naive();

    spawn(|| {
        trace!("thread 0");

        for i in 0..32 {
            spawn(move || trace!("thread {}", i + 1)).unwrap();
        }
    })?;

    unsafe { task::enter() };
    // for _ in 0..512 {
    //     thread::Builder::new(AddrSpace::Kernel)
    //         .stack_size(8192)
    //         .spawn_raw(per_thread, ptr::null_mut())?;
    // }

    // loop {
    //     thread::park_or_wait();
    // }
    // Ok(())
}

extern "C" fn cpu_start(smp_info: *const LimineSmpInfo) -> ! {
    unsafe {
        let thrd = (*smp_info).processor_id as usize;
        hal::task::init_hw_thread(thrd);
        task::enter();
    }

    trace!("cpu msr = {}", unsafe { hw_thread_id() });

    unsafe { interrupts::enable() };

    loop {
        yield_now();
    }
}

#[track_caller]
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    error!("thread {:?} {}", task::current(), panic_info);
    loop {
        unsafe { interrupts::wait() };
    }
}

macro_rules! dbg {
    ($v:expr) => {{
        ::log::info!("[{}:{}] {} := {:#?}", file!(), line!(), stringify!($v), $v);
        $v
    }};
}

pub(crate) use dbg;
