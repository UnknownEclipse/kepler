use core::{
    cell::{Cell, UnsafeCell},
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, AtomicU16, Ordering},
};

const PAGE_SIZE: usize = 1 << 16;

#[repr(C, align(65536))]
#[derive(Debug)]
pub struct Page {
    free: u32,
    local_free_head: Cell<Option<NonNull<Block>>>,
    local_free_tail: Cell<Option<NonNull<Block>>>,
    remote_free: AtomicPtr<Block>,
    block_size: u16,
    buffer: UnsafeCell<[u8; PAGE_SIZE - 10]>,
}

impl Page {
    /// # Safety
    /// 1. This must be called from the page's owning thread
    pub unsafe fn allocate(&self) -> Option<NonNull<[u8]>> {
        let head = self.local_free.get()?;

        debug_assert!(head.as_ptr().is_aligned_to(self.block_size.into()));
        debug_assert!(head.as_ptr().is_aligned());
        // debug_assert!(self.buffer.get().as_ptr_range().contains(&head.as_ptr()));

        unsafe {
            // SAFETY: Head pointer points within this page's memory buffer and is
            // aligned to the block size of this page.
            // self.local_free.set(head.as_ref().next.get())
        };

        Some(NonNull::slice_from_raw_parts(
            head.cast(),
            self.block_size.into(),
        ))
    }

    /// # Safety
    /// The provided pointer must point within this page and be aligned to the page
    /// block size.
    pub unsafe fn deallocate(&self, ptr: NonNull<u8>) {
        debug_assert!(ptr.as_ptr().is_aligned_to(self.block_size.into()));
    }

    pub unsafe fn local_merge_free(&self) {
        let head = self.remote_free.swap(ptr::null_mut(), Ordering::AcqRel);

        let old_tail = self.local_free_tail.get();
        if let Some(old_tail) = old_tail {
            old_tail.as_ref().next.store(head, Ordering::Release);
        }
    }
}

unsafe impl Sync for Page {}

#[repr(transparent)]
#[derive(Debug)]
struct Block {
    next: AtomicPtr<Block>,
}
