#![no_std]
#![no_main]
#![feature(
    abi_x86_interrupt,
    allocator_api,
    slice_ptr_get,
    nonnull_slice_from_raw_parts,
    never_type,
    naked_functions
)]

extern crate alloc;

use alloc::string::String;
use core::{panic::PanicInfo, ptr, sync::atomic::AtomicPtr};

use hal::{
    intrin::halt,
    task::{context_switch, Context},
};
use lagrange::thread;
use limine::{LimineHhdmRequest, LimineMemmapRequest, LimineMemoryMapEntryType};
use pci::types::{ClassId, SubclassId};
use tracing::{info_span, subscriber::set_global_default};
use x86_64::{registers::control::Cr3, structures::paging::OffsetPageTable};

use crate::{
    stdout::{init_logger, serial},
    subscriber::KernelSubscriber,
};

mod allocator;
mod idt;
mod irq_mutex;
mod random;
mod stdout;
mod subscriber;
mod vm;

static MEMORY_MAP: LimineMemmapRequest = LimineMemmapRequest::new(0);
static HHDM: LimineHhdmRequest = LimineHhdmRequest::new(0);

// #[global_allocator]
// static HEAP: LockedHeap = LockedHeap::empty();

#[no_mangle]
extern "C" fn _start() -> ! {
    let subscriber = KernelSubscriber::new();
    set_global_default(subscriber).unwrap();

    {
        let span = info_span!("span");
        let _guard = span.enter();
        tracing::info!("Hello from tracing!");
    }

    init_logger().expect("log init failed");
    log::set_max_level(log::LevelFilter::Trace);

    log::info!("Hello, world!");

    idt::init();

    log::info!("initialized idt");

    let hhdm_offset = HHDM
        .get_response()
        .get()
        .expect("higher half mapping failed")
        .offset;

    let entries = MEMORY_MAP
        .get_response()
        .get()
        .unwrap()
        .memmap()
        .iter()
        .filter_map(|entry| {
            if entry.typ != LimineMemoryMapEntryType::Usable {
                None
            } else {
                Some((entry.base, entry.len))
            }
        });

    vm::init(entries);
    allocator::init();

    tracing::info!("tracing initialized");

    let s = String::from("Hi!");
    log::info!("allocated string: {}", s);

    let mut nvme = pci::enumerate(pci::DefaultConfigSpace)
        .filter(|dev| dev.class_id() == (ClassId(1), SubclassId(8)));

    let (l4, _) = Cr3::read();
    let l4 = l4.start_address().as_u64() + hhdm_offset;

    let mapper = unsafe {
        let l4 = &mut *(l4 as usize as *mut x86_64::structures::paging::PageTable);
        OffsetPageTable::new(l4, x86_64::VirtAddr::new(hhdm_offset))
    };

    if let Some(nvme) = nvme.next() {
        log::info!("found nvme device: {:#x?}", nvme);
        let bar0: u64 = nvme.read(0x10).into();
        let bar1: u64 = nvme.read(0x14).into();

        assert_ne!(bar0 & 0b100, 0);
        let addr = (bar0 & 0xfffffff0) + ((bar1 & 0xffffffff) << 32);

        log::info!("{:#x?}", addr);
    }

    log::info!("DONE!");

    let t0 = thread::spawn(|| {
        log::info!("thread #0 started");
        for _ in 0..20 {
            unsafe {
                halt();
            }
        }
        log::info!("thread #0 finished");
        String::from("Finished")
    });

    log::info!("hi");
    thread::yield_now();
    log::info!("bye");

    assert_eq!(t0.join(), String::from("Finished"));
    log::info!("bye 2");

    loop {
        unsafe { halt() };
        thread::yield_now();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial::println!("{}", info);
    loop {
        unsafe { halt() };
    }
}

extern "C" fn start_handler(ptr: *mut ()) -> ! {
    let data: *mut ThreadState = ptr.cast();

    log::info!("hello from thread <main>");

    let mut saved = ptr::null_mut();
    unsafe { context_switch(&mut saved, (*data).boot_thread_ctx) };

    loop {
        unsafe { halt() }
    }
}

struct ThreadState {
    boot_thread_ctx: *mut Context,
}

static CURRENT_THREAD: AtomicPtr<ThreadState> = AtomicPtr::new(ptr::null_mut());

unsafe fn start_thread(
    data: *mut (),
    f: extern "C" fn(*mut ()) -> !,
    saved_state: *mut *mut Context,
) {
    let stack_slice = vm::alloc_pages(128).unwrap();
    let stack = stack_slice.as_ptr() as *mut u8;
    let top = stack.add(stack_slice.len());
    let sp: *mut Context = top.cast();
    let sp = sp.sub(1);

    let initial_context = Context::with_initial(f, data);
    sp.write(initial_context);

    // let rip = rip.cast();
    // ptr::write(rip, initial_context);
    unsafe { context_switch(saved_state, sp) };
}
