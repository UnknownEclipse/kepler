use core::{
    alloc::{Allocator, GlobalAlloc, Layout},
    ptr::{self, NonNull},
};

#[derive(Debug)]
pub struct WrapAsGlobal<A> {
    allocator: A,
}

impl<A> WrapAsGlobal<A> {
    pub const fn new(allocator: A) -> Self {
        Self { allocator }
    }

    pub fn get(&self) -> &A {
        &self.allocator
    }
}

unsafe impl<A> GlobalAlloc for WrapAsGlobal<A>
where
    A: Allocator,
{
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.allocator
            .allocate(layout)
            .map(|v| v.as_mut_ptr())
            .unwrap_or(ptr::null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        if let Some(ptr) = NonNull::new(ptr) {
            self.allocator.deallocate(ptr, layout);
        }
    }

    unsafe fn alloc_zeroed(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.allocator
            .allocate_zeroed(layout)
            .map(|v| v.as_mut_ptr())
            .unwrap_or(ptr::null_mut())
    }

    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: core::alloc::Layout,
        new_size: usize,
    ) -> *mut u8 {
        let ptr = NonNull::new_unchecked(ptr);
        let old_layout = layout;
        let new_layout = Layout::from_size_align_unchecked(new_size, layout.size());

        let result = if layout.size() < new_size {
            self.allocator.grow(ptr, old_layout, new_layout)
        } else {
            self.allocator.shrink(ptr, old_layout, new_layout)
        };

        result.map(|v| v.as_mut_ptr()).unwrap_or(ptr::null_mut())
    }
}
