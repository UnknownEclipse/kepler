use alloc::vec::Vec;
use core::{iter::Step, num::NonZeroUsize, ops::Range};

use hal::{
    interrupts,
    vm_types::{Frame, FrameAllocError, FrameAllocator, PhysAddr, VirtAddr},
};
use limine::LimineMemoryMapEntryType;
use log::trace;
use spin::{mutex::SpinMutex, Once};

use super::{HHDM_REQUEST, MMAP_REQUEST};

#[derive(Debug)]
pub struct Global;

unsafe impl FrameAllocator for Global {
    fn allocate_frame(&self) -> Result<Frame, hal::vm_types::FrameAllocError> {
        let locked = GLOBAL.get().ok_or(FrameAllocError)?;
        interrupts::without(|_| locked.lock().pop()).ok_or(FrameAllocError)
    }

    unsafe fn deallocate_frame(&self, frame: Frame) {
        let locked = GLOBAL.get().expect("frame allocator not initialized");
        interrupts::without(|_| locked.lock().push(frame));
    }

    fn allocate_contiguous_frames(
        &self,
        _n: usize,
    ) -> Result<Range<Frame>, hal::vm_types::FrameAllocError> {
        unimplemented!()
    }

    unsafe fn deallocate_contiguous_frames(&self, _frames: Range<Frame>) {
        unimplemented!()
    }
}

pub fn init() {
    trace!("beginning initialization");
    GLOBAL.call_once(|| SpinMutex::new(build_global()));
    trace!("finished initialization");
}

pub fn hhdm_end() -> VirtAddr {
    let mmap_response = MMAP_REQUEST
        .get_response()
        .get()
        .expect("memory map request failed");

    let hhdm_response = HHDM_REQUEST
        .get_response()
        .get()
        .expect("higher-half direct mapping failed");

    let base = hhdm_response.offset;

    let last = mmap_response.memmap().last().unwrap();
    let end = last.base + last.len;

    VirtAddr::from_usize((end + base) as usize)
}

static GLOBAL: Once<SpinMutex<DirectlyMappedLinkedStack>> = Once::new();

fn build_global() -> DirectlyMappedLinkedStack {
    let mmap_response = MMAP_REQUEST
        .get_response()
        .get()
        .expect("memory map request failed");

    let memory_map = mmap_response
        .memmap()
        .iter()
        .filter(|entry| entry.typ == LimineMemoryMapEntryType::Usable)
        .map(|entry| base_len_to_frame_range(entry.base, entry.len))
        .collect();

    let hhdm_response = HHDM_REQUEST
        .get_response()
        .get()
        .expect("higher-half direct mapping failed");

    let phys_base = VirtAddr::from_usize(hhdm_response.offset as usize);

    DirectlyMappedLinkedStack {
        head: None,
        memory_map,
        phys_base,
    }
}

fn base_len_to_frame_range(base: u64, len: u64) -> Range<Frame> {
    let start = PhysAddr::from_usize(base as usize);
    let end = PhysAddr::from_usize((base + len) as usize);
    let start = Frame::from_base(start).unwrap();
    let end = Frame::from_base(end).unwrap();
    start..end
}

#[derive(Debug)]
struct DirectlyMappedLinkedStack {
    /// The head of the stack
    head: Option<NonZeroUsize>,
    phys_base: VirtAddr,
    /// The memory map given by the bootloader. This will slowly be consumed when no
    /// entries are left in the stack.
    memory_map: Vec<Range<Frame>>,
}

impl DirectlyMappedLinkedStack {
    pub fn pop(&mut self) -> Option<Frame> {
        self.pop_stack().or_else(|| self.pop_from_memory_map())
    }

    pub unsafe fn push(&mut self, frame: Frame) {
        unsafe {
            *self.map_addr(frame.addr()).cast() = self.head;
        }
        self.head = NonZeroUsize::new(frame.addr().as_usize().wrapping_shr(12) + 1);
    }

    fn pop_stack(&mut self) -> Option<Frame> {
        let frame = self.head?.get() - 1;
        let addr = PhysAddr::from_usize(frame << 12);
        let frame = Frame::from_base(addr)?;
        unsafe {
            self.head = *self.map_addr(addr).cast();
        }
        Some(frame)
    }

    fn pop_from_memory_map(&mut self) -> Option<Frame> {
        loop {
            let last = self.memory_map.last_mut()?;
            if last.is_empty() {
                self.memory_map.pop();
                continue;
            }
            let frame = last.start;
            last.start = Step::forward(frame, 1);
            return Some(frame);
        }
    }

    fn map_addr(&self, addr: PhysAddr) -> *mut u8 {
        self.phys_base.as_ptr::<u8>().wrapping_add(addr.as_usize())
    }
}
