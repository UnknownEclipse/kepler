use core::{
    alloc::{AllocError, Allocator, Layout},
    ptr::NonNull,
};

#[derive(Debug)]
pub struct NopAllocator;

unsafe impl Allocator for NopAllocator {
    fn allocate(&self, _layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        Err(AllocError)
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
        unreachable!("memory is not be allocated, and so should not be freed");
    }
}
