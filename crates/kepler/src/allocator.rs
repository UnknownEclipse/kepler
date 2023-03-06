use core::{
    alloc::{AllocError, Allocator, GlobalAlloc, Layout},
    ptr::{self, NonNull},
};

use divvy::{global::WrapAsGlobal, hybrid::HybridAllocator};
use linked_list_allocator::LockedHeap;

use crate::vm;

#[global_allocator]
static ALLOCATOR: WrapAsGlobal<HybridAllocator<LinkedListAllocator, 8192>> =
    WrapAsGlobal::new(HybridAllocator::new());

pub fn init() {
    let heap_region = vm::alloc_pages(64).unwrap();
    let heap = unsafe { LockedHeap::new(heap_region.as_mut_ptr(), heap_region.len()) };

    ALLOCATOR
        .get()
        .try_init_primary(LinkedListAllocator(heap))
        .ok();
}

struct LinkedListAllocator(LockedHeap);

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
