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
use core::{
    arch::global_asm,
    mem,
    panic::PanicInfo,
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

use hal::{
    intrin::{breakpoint, halt},
    task::{context_switch, Context},
};
use limine::{
    LimineEfiSystemTableRequest, LimineHhdmRequest, LimineMemmapRequest, LimineMemoryMapEntryType,
    LimineRsdpRequest,
};
use linked_list_allocator::LockedHeap;
use nvme::controller_attributes::ControllerAttributes;
use pci::types::{ClassId, SubclassId};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{Mapper, OffsetPageTable, PhysFrame},
};

use crate::stdout::{init_logger, serial};

mod executor;
mod idt;
mod irq_mutex;
mod random;
mod stdout;
mod task;
mod vm;

static MEMORY_MAP: LimineMemmapRequest = LimineMemmapRequest::new(0);
static SYSTEM_TABLE: LimineEfiSystemTableRequest = LimineEfiSystemTableRequest::new(0);
static RSDP: LimineRsdpRequest = LimineRsdpRequest::new(0);
static HHDM: LimineHhdmRequest = LimineHhdmRequest::new(0);

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

#[no_mangle]
extern "C" fn _start() -> ! {
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

    {
        let heap_region = vm::alloc_pages(64).unwrap();
        let mut heap = HEAP.lock();
        unsafe { heap.init(heap_region.as_mut_ptr(), heap_region.len()) };
    }

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

        // let attrs = unsafe { &*((addr + 0x100) as usize as *mut ControllerAttributes) };
        // log::info!("{:?}", attrs);
    }

    log::info!("DONE!");

    let mut state = ThreadState {
        boot_thread_ctx: ptr::null_mut(),
    };
    CURRENT_THREAD.store(&mut state, Ordering::Relaxed);

    unsafe { start_thread(ptr::null_mut(), start_handler, &mut state.boot_thread_ctx) };

    log::info!("hello from thread <boot>");

    loop {
        unsafe { halt() };
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial::println!("{}", info);
    loop {
        unsafe { halt() };
    }
}

#[derive(Debug, Clone, Copy)]
struct AcpiHandler;

impl acpi::AcpiHandler for AcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<Self, T> {
        // for page in
        let vm = holo::Global;
        todo!()
    }

    fn unmap_physical_region<T>(region: &acpi::PhysicalMapping<Self, T>) {
        todo!()
    }
}

extern "C" fn start_handler() -> ! {
    let data = CURRENT_THREAD.load(Ordering::Relaxed);

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

unsafe fn start_thread(data: *mut u8, f: extern "C" fn() -> !, saved_state: *mut *mut Context) {
    let stack_slice = vm::alloc_pages(128).unwrap();
    let stack = stack_slice.as_ptr() as *mut u8;
    let top = stack.add(stack_slice.len());
    let sp: *mut Context = top.cast();
    let sp = sp.sub(1);

    let initial_context = Context::with_target(f);
    sp.write(initial_context);

    // let rip = rip.cast();
    // ptr::write(rip, initial_context);
    unsafe { context_switch(saved_state, sp) };
}

struct Thread {
    context: *mut Context,
}
