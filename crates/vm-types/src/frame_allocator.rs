use core::ops::Range;

use crate::Frame;

#[derive(Debug)]
pub struct FrameAllocError;

/// # Safety
/// Implementors of this trait need to adhere to very strict safety guidelines. The
/// virtual memory system is one of the key parts of any OS, so bugs here can have
/// disastrous effects.
pub unsafe trait FrameAllocator {
    fn allocate_frame(&self) -> Result<Frame, FrameAllocError>;
    /// # Safety
    /// 1. The frame must be valid and unused
    unsafe fn deallocate_frame(&self, frame: Frame);

    fn allocate_contiguous_frames(&self, n: usize) -> Result<Range<Frame>, FrameAllocError>;
    /// # Safety
    /// 1. Every frame in the range must exist and be unused
    unsafe fn deallocate_contiguous_frames(&self, frames: Range<Frame>);
}
