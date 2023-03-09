use vm_types::{FrameAllocError, FrameAllocator};

use super::stack::LockedStack;

pub struct HybridPhysicalAllocator {
    stack: LockedStack,
}

unsafe impl FrameAllocator for HybridPhysicalAllocator {
    fn allocate_frame(&self) -> Result<vm_types::Frame, FrameAllocError> {
        if let Ok(frame) = self.stack.allocate_frame() {
            return Ok(frame);
        }
        Err(FrameAllocError)
    }

    unsafe fn deallocate_frame(&self, frame: vm_types::Frame) {
        todo!()
    }

    fn allocate_contiguous_frames(
        &self,
        n: usize,
    ) -> Result<core::ops::Range<vm_types::Frame>, FrameAllocError> {
        todo!()
    }

    unsafe fn deallocate_contiguous_frames(&self, frames: core::ops::Range<vm_types::Frame>) {
        todo!()
    }
}
