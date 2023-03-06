use core::{
    alloc::{AllocError, Allocator, Layout},
    ptr::NonNull,
};

use spin::Once;

use crate::bump::BumpAllocator;

/// A hybrid allocator that will use a given allocator if available, or fall back to
/// a bump allocator.
///
/// This is intended to be used in embedded situations where the necessary infrastructure
/// for a more advanced allocator is not available immediately, such as the early
/// boot process of a kernel before virtual memory allocation is set up.
#[derive(Debug)]
pub struct HybridAllocator<A, const N: usize> {
    allocator: Once<A>,
    bootstrap: BumpAllocator<N>,
}

impl<A, const N: usize> HybridAllocator<A, N> {
    pub const fn new() -> Self {
        Self {
            allocator: Once::new(),
            bootstrap: BumpAllocator::new(),
        }
    }

    pub fn try_init_primary(&self, primary: A) -> Result<(), A> {
        let mut primary = Some(primary);
        self.allocator.call_once(|| primary.take().unwrap());
        match primary {
            Some(a) => Err(a),
            None => Ok(()),
        }
    }
}

unsafe impl<A, const N: usize> Allocator for HybridAllocator<A, N>
where
    A: Allocator,
{
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        if let Some(a) = self.allocator.get() {
            a.allocate(layout)
        } else {
            self.bootstrap.allocate(layout)
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        if let Some(a) = self.allocator.get() {
            a.deallocate(ptr, layout);
        } else {
            self.bootstrap.deallocate(ptr, layout);
        }
    }

    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        if let Some(a) = self.allocator.get() {
            a.allocate_zeroed(layout)
        } else {
            self.bootstrap.allocate_zeroed(layout)
        }
    }

    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() >= old_layout.size(),
            "`new_layout.size()` must be greater than or equal to `old_layout.size()`"
        );

        if let Some(a) = self.allocator.get() {
            a.grow(ptr, old_layout, new_layout)
        } else {
            self.bootstrap.grow(ptr, old_layout, new_layout)
        }
    }

    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() >= old_layout.size(),
            "`new_layout.size()` must be greater than or equal to `old_layout.size()`"
        );

        if let Some(a) = self.allocator.get() {
            a.grow_zeroed(ptr, old_layout, new_layout)
        } else {
            self.bootstrap.grow_zeroed(ptr, old_layout, new_layout)
        }
    }

    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() <= old_layout.size(),
            "`new_layout.size()` must be smaller than or equal to `old_layout.size()`"
        );

        if let Some(a) = self.allocator.get() {
            a.shrink(ptr, old_layout, new_layout)
        } else {
            self.bootstrap.shrink(ptr, old_layout, new_layout)
        }
    }
}
