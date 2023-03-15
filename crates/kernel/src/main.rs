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
    vec_push_within_capacity,
    maybe_uninit_uninit_array,
    const_maybe_uninit_uninit_array,
    slice_ptr_get,
    naked_functions,
    ptr_as_uninit
)]

extern crate alloc;

use alloc::{sync::Arc, vec::Vec};
use core::{arch::asm, mem, panic::PanicInfo, ptr};

use arch::cpu;
use error::KernResult;
use hal::{interrupts, vm_types::MapOptions, x86_64::instr::int3};
use limine::{LimineSmpInfo, LimineSmpRequest};
use log::{error, info, set_logger, set_max_level, trace, LevelFilter};
use stdio::StdoutLogger;
use tracing::instrument;
use x86_64::registers::model_specific::Msr;

use crate::{
    arch::x86_64::gdt,
    memory::allocate_user,
    sync::{barrier::Barrier, Mutex},
    task::{spawn, yield_now},
};

mod arch;
mod cpu_local;
mod error;
mod memory;
mod process;
mod random;
mod stdio;
mod subscriber;
mod sync;
mod syscall;
mod task;

static SMP_REQUEST: LimineSmpRequest = LimineSmpRequest::new(0);

#[no_mangle]
pub extern "C" fn _start() -> ! {
    kernel_main().expect("kernel error occurred");

    loop {
        unsafe { interrupts::wait() };
    }
}

fn kernel_main() -> KernResult<()> {
    set_logger(&StdoutLogger).expect("no other logger is set");
    set_max_level(LevelFilter::Trace);

    info!("kepler v0.1.0");
    info!("beginning initialization");

    unsafe {
        // These submodules all use Lazy/Once internally, however we choose to
        // explicitly initialize them here to make the ordering consistent and prevent
        // any kind of associated bugs.

        arch::init();
        // interrupt_table::init();
        memory::init()?;
        hal::task::init_hw_thread(0);
    }

    info!("finished initialization");

    // task::init_naive_scheduler();

    // task::init_naive_smp_scheduler(smp_response.cpu_count as usize);
    // task::init_naive_scheduler();
    // tracing::subscriber::set_global_default(KernelSubscriber::default()).unwrap();

    // tracing::trace!("Hello tracing!");
    let stack = allocate_user(8192).unwrap();
    let top = (stack.as_ptr() as *mut u8).wrapping_add(stack.len());

    let user_memory = allocate_user(8192).unwrap();
    unsafe { user_memory.as_mut_ptr().write(10) };
    unsafe {
        ptr::copy(
            ring3_entry as usize as *mut u8,
            user_memory.as_mut_ptr(),
            4096,
        )
    };
    let addr = user_memory.as_ptr() as *mut u8;
    info!("user stack start: {:p}", stack);
    info!("user code start: {:p}", user_memory);
    trace!("addr = {:p}", addr);

    let mut efer = Msr::new(0xc0000080);
    let mut star = Msr::new(0xc0000081);
    let mut lstar = Msr::new(0xc0000082);
    let mut cstar = Msr::new(0xc0000083);
    let mut sfmask = Msr::new(0xc0000084);

    unsafe {
        lstar.write(0x0);

        let enable = efer.read() | 1;
        efer.write(enable);

        let cs: u64 = gdt::selectors().user_code_selector.0.into();
        let ss: u64 = gdt::selectors().user_data_selector.0.into();

        star.write((cs << 48) | (ss << 46));
    }

    unsafe {
        asm!(
            "
            mov rsp, {}
            mov rcx, {}
            mov r11, 0x202
            sysret",
            in(reg) top,
            in(reg) addr,
            options(noreturn)
        );
    }

    // foo(5);
    // spawn(|| {
    //     trace!("thread 0");

    //     for i in 0..32 {
    //         spawn(move || trace!("thread {}", i + 1)).unwrap();
    //     }
    // })?;

    // let user_stack = allocate_user_stack(8192)?;
    // let user_stack_ptr = unsafe {
    //     user_stack
    //         .as_uninit_slice_mut()
    //         .as_mut_ptr_range()
    //         .end
    //         .sub(1)
    // };

    // unsafe {
    //     sysret(
    //         VirtAddr::from_usize(ring3_entry as usize),
    //         VirtAddr::from_ptr(user_stack_ptr),
    //         0x202,
    //     )
    // };

    // let value = Arc::new((Mutex::new(Vec::new()), Barrier::new(6)));

    // // Ok(())
    // for thread in 0..300 {
    //     let shared = value.clone();
    //     spawn(move || {
    //         trace!("thread {} entry", task::current());
    //         let (mtx, barrier) = &*shared;
    //         let mut guard = mtx.lock();
    //         for i in 0..5 {
    //             yield_now();
    //             guard.push(i);
    //         }
    //         trace!("thread {} finished", task::current());
    //         mem::drop(guard);
    //         barrier.wait();
    //     })?;
    // }

    // spawn(move || {
    //     value.1.wait();
    //     trace!("finished!");
    // })?;

    // for cpu in smp_response.cpus() {
    //     cpu.goto_address = apu_start;
    // }

    // unsafe { task::enter() };
    // for _ in 0..512 {
    //     thread::Builder::new(AddrSpace::Kernel)
    //         .stack_size(8192)
    //         .spawn_raw(per_thread, ptr::null_mut())?;
    // }

    // loop {
    //     thread::park_or_wait();
    // }
    // Ok(())
    Ok(())
}

#[track_caller]
#[panic_handler]
fn panic(panic_info: &PanicInfo) -> ! {
    error!("{}", panic_info);
    loop {
        unsafe { interrupts::wait() };
    }
}

#[instrument]
fn foo(val: i32) {
    tracing::info!("hello again");
}

#[no_mangle]
pub extern "C" fn ring3_entry() -> ! {
    // Should fault
    unsafe { interrupts::disable() };
    loop {
        unsafe { int3() };
    }
}

extern "C" fn apu_start(info: *const LimineSmpInfo) -> ! {
    unsafe {
        let id = (*info).processor_id as usize;
        cpu::init(id);
        info!("apu start: {}", (*info).processor_id);
    };

    unsafe { task::enter() };
}
