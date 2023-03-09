use core::{
    alloc::{AllocError, Allocator, GlobalAlloc, Layout},
    iter::Step,
    ptr::{self, NonNull},
};

use divvy::{global::WrapAsGlobal, hybrid::HybridAllocator};
use hal::interrupts;
use linked_list_allocator::LockedHeap;
use log::trace;

use super::{KERNEL_ADDRESS_SPACE, PAGE_SIZE};

const HEAP_SIZE: usize = 1 << 18;

pub fn init() {
    trace!("beginning initialization");
    interrupts::without(|_| {
        let mut addrspace = KERNEL_ADDRESS_SPACE.lock();

        trace!("allocating kernel heap...");
        let region = addrspace
            .allocate_virtual_region(HEAP_SIZE, 0)
            .expect("failed to kernel heap");

        let ptr = region.start.addr().as_ptr();
        let len = Step::steps_between(&region.start, &region.end).unwrap() * PAGE_SIZE;
        let region = ptr::slice_from_raw_parts_mut(ptr, len);
        let region = NonNull::new(region).unwrap();

        trace!("finished allocating kernel heap: {:#p}", region);

        trace!("initializing primary allocator...");
        unsafe {
            ALLOCATOR
                .get()
                .try_init_primary(LinkedListAllocator::new(region))
                .unwrap_or_else(|_| panic!("failed to initialize allocator"));
        }
        trace!("finished initializing primary allocator");
    });
    trace!("finished initialization");
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
