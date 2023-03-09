use core::{fmt::Debug, iter::Step, ops::Range};

use spin::mutex::SpinMutex;
use vm_types::{Frame, FrameAllocError, FrameAllocator, PhysAddr};

#[derive(Debug)]
pub struct LockedStack {
    inner: SpinMutex<Stack>,
}

unsafe impl FrameAllocator for LockedStack {
    fn allocate_frame(&self) -> Result<Frame, vm_types::FrameAllocError> {
        self.inner.lock().pop().ok_or(FrameAllocError)
    }

    unsafe fn deallocate_frame(&self, frame: Frame) {
        self.inner
            .lock()
            .push(frame)
            .expect("no error should occur here");
    }

    fn allocate_contiguous_frames(
        &self,
        n: usize,
    ) -> Result<Range<Frame>, vm_types::FrameAllocError> {
        match n {
            0 => Ok(Frame::zero()..Frame::zero()),
            1 => {
                let start = self.allocate_frame()?;
                let end = Frame::forward(start, 1);
                Ok(start..end)
            }
            _ => panic!("stack only supports single frame allocations"),
        }
    }

    unsafe fn deallocate_contiguous_frames(&self, frames: Range<Frame>) {
        if !frames.is_empty() {
            self.deallocate_frame(frames.start);
        }
    }
}

pub struct Stack {
    buf: *mut [PhysAddr],
    len: usize,
}

impl Debug for Stack {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let slice = unsafe { &(*self.buf)[..self.len] };
        f.debug_struct("Stack").field("items", &slice).finish()
    }
}

impl Stack {
    pub unsafe fn new(buf: *mut [PhysAddr]) -> Self {
        Self { buf, len: 0 }
    }

    pub fn pop(&mut self) -> Option<Frame> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            let addr = unsafe { (*self.buf)[self.len] };
            Frame::from_base(addr)
        }
    }

    pub fn push(&mut self, frame: Frame) -> Result<(), Frame> {
        if self.len == self.buf.len() {
            Err(frame)
        } else {
            unsafe {
                (*self.buf)[self.len] = frame.addr();
            }
            self.len += 1;
            Ok(())
        }
    }
}
