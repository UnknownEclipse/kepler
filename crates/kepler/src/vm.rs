use core::{
    alloc::AllocError,
    ptr::{self, NonNull},
};

use spin::mutex::SpinMutex;
use staticvec::StaticVec;

static REGIONS: SpinMutex<StaticVec<(u64, u64), 64>> = SpinMutex::new(StaticVec::new());

pub fn init<I>(regions: I)
where
    I: IntoIterator<Item = (u64, u64)>,
{
    let mut guard = REGIONS.lock();
    for region in regions {
        if guard.try_push(region).is_err() {
            break;
        }
    }
}

pub fn alloc_pages(pages: usize) -> Result<NonNull<[u8]>, AllocError> {
    let mut guard = REGIONS.lock();
    while let Some((addr, size)) = guard.last_mut().copied() {
        if size == 0 {
            guard.pop();
            continue;
        }
        let len = (pages * page_size()) as u64;
        if size < len {
            return Err(AllocError);
        }
        let ptr = addr + (size - len);
        guard.last_mut().unwrap().1 = size - len;
        return Ok(NonNull::new(ptr::slice_from_raw_parts_mut(
            ptr as usize as *mut u8,
            len as usize,
        ))
        .unwrap());
    }
    Err(AllocError)
}

pub unsafe fn dealloc_pages(pages: NonNull<[u8]>) {
    log::warn!(
        "deallocating memory at address={:?}, size={:#x}",
        pages.as_ptr(),
        pages.len()
    );
}

#[inline]
pub fn page_size() -> usize {
    4096
}
