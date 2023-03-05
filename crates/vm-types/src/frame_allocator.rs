use core::ops::Range;

use crate::Frame;

#[derive(Debug)]
pub struct PhysAllocError;

pub unsafe trait PhysicalMemoryAllocator {
    fn allocate_frame(&self) -> Result<Frame, PhysAllocError>;
    unsafe fn deallocate_frame(&self, frame: Frame);

    fn allocate_contiguous_frames(&self, n: usize) -> Result<Range<Frame>, PhysAllocError>;
    unsafe fn deallocate_contiguous_frames(&self, frames: Range<Frame>);
}
