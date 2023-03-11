use core::{
    alloc::{AllocError, Allocator, GlobalAlloc, Layout},
    ptr::{self, NonNull},
};

use divvy::{global::WrapAsGlobal, hybrid::HybridAllocator};
use linked_list_allocator::LockedHeap;
use log::trace;

use crate::{
    error::KernResult,
    memory::{AddrSpace, AllocOptions},
};

const HEAP_SIZE: usize = 1 << 18;

pub unsafe fn init() -> KernResult<()> {
    trace!("beginning initialization");

    trace!("allocating kernel heap...");
    let region = AllocOptions::new(HEAP_SIZE)
        .start_guard_pages(1)
        .end_guard_pages(1)
        .allocate_in_address_space(&AddrSpace::Kernel)?;

    trace!("finished allocating kernel heap: {:#p}", region);

    trace!("initializing primary allocator...");

    ALLOCATOR
        .get()
        .try_init_primary(LinkedListAllocator::new(region))
        .unwrap_or_else(|_| panic!("failed to initialize allocator"));

    trace!("finished initializing primary allocator");
    trace!("finished initialization");
    Ok(())
}

#[global_allocator]
static ALLOCATOR: WrapAsGlobal<HybridAllocator<LinkedListAllocator, 8192>> =
    WrapAsGlobal::new(HybridAllocator::new());

struct LinkedListAllocator(LockedHeap);

impl LinkedListAllocator {
    pub unsafe fn new(region: NonNull<[u8]>) -> Self {
        Self(LockedHeap::new(region.as_ptr() as *mut u8, region.len()))
    }
}

unsafe impl Allocator for LinkedListAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        unsafe {
            let ptr = self.0.alloc(layout);
            if ptr.is_null() {
                return Err(AllocError);
            }
            let slice = ptr::slice_from_raw_parts_mut(ptr, layout.size());
            Ok(NonNull::new(slice).unwrap())
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.0.dealloc(ptr.as_ptr(), layout);
    }
}
