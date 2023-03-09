use core::ops::Range;

use crate::Frame;

#[derive(Debug)]
pub struct FrameAllocError;

pub unsafe trait FrameAllocator {
    fn allocate_frame(&self) -> Result<Frame, FrameAllocError>;
    unsafe fn deallocate_frame(&self, frame: Frame);

    fn allocate_contiguous_frames(&self, n: usize) -> Result<Range<Frame>, FrameAllocError>;
    unsafe fn deallocate_contiguous_frames(&self, frames: Range<Frame>);
}
