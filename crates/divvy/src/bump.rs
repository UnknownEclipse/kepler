use core::{
    alloc::{AllocError, Allocator, Layout},
    cell::SyncUnsafeCell,
    mem::MaybeUninit,
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Debug)]
pub struct BumpAllocator<const N: usize> {
    pos: AtomicUsize,
    array: SyncUnsafeCell<[MaybeUninit<u8>; N]>,
}

impl<const N: usize> BumpAllocator<N> {
    pub const fn new() -> Self {
        Self {
            pos: AtomicUsize::new(0),
            array: SyncUnsafeCell::new(MaybeUninit::uninit_array()),
        }
    }

    fn buffer(&self) -> NonNull<[u8]> {
        let ptr: *mut [u8; N] = self.array.get().cast();
        NonNull::new(ptr).unwrap()
    }
}

unsafe impl<const N: usize> Allocator for BumpAllocator<N> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let slice = self.buffer();
        let base = slice.as_mut_ptr() as usize;
        let mut pos = self.pos.load(Ordering::Acquire);

        loop {
            let current = base + pos;
            let current_aligned = align_up(current, layout.align());
            let start = current_aligned - base;
            let end = start.checked_add(layout.size()).ok_or(AllocError)?;

            if slice.len() < end {
                return Err(AllocError);
            }

            match self
                .pos
                .compare_exchange(pos, end, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => return Ok(unsafe { slice.get_unchecked_mut(start..end) }),
                Err(v) => {
                    pos = v;
                }
            }
        }
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {}

    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        _old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        Ok(NonNull::slice_from_raw_parts(ptr, new_layout.size()))
    }
}

fn align_up(v: usize, align: usize) -> usize {
    ((v + align - 1) / align) * align
}
