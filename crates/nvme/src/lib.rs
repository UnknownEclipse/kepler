#![no_std]

use core::{marker::PhantomData, mem, ops::Range};

use controller_attributes::ControllerAttributes;
use hal_core::volatile::Volatile;

pub mod controller_attributes;
pub mod cqe;
pub mod sqe;

#[derive(Debug)]
pub struct Nvme<H> {
    handler: H,
    registers: &'static mut Registers,
}

#[derive(Debug)]
pub enum NvmeError {
    VirtualMemoryError,
}

/// Provide access to virtual memory operations needed by the nvme driver.
///
/// # Safety
/// 1. Implementors of this trait must have sound virtual memory systems.
pub unsafe trait NvmeHandler {
    /// # Safety
    /// 1. The frame number must be valid and unused
    unsafe fn map_frame(&mut self, phys: u64) -> Result<*mut u8, NvmeError>;

    /// # Safety
    /// 1. The virtual address must point to a valid, unused page.
    unsafe fn unmap_page(&mut self, virt: *mut u8);

    fn allocate_contiguous_frames(&mut self, n: usize) -> Result<Range<u64>, NvmeError>;

    /// # Safety
    /// 1. The range of frames must be valid and unused.
    unsafe fn deallocate_frames(&mut self, frames: Range<u64>);
}

pub struct AdminSq {}

pub struct AdminCq {}

pub trait Command {
    type CommandSet;
    type Sqe;
    type Cqe;
}

pub trait SubmissionQueueEntry {
    type CommandSet;

    fn to_raw(&self) -> RawSqe;
}

pub trait CompletionQueueEntry {
    type CommandSet;

    fn to_raw(&self) -> RawCqe;
}

pub struct RawSubmissionQueue {}

impl RawSubmissionQueue {
    pub fn push(&mut self, sqe: RawSqe) -> Result<(), RawSqe> {
        todo!()
    }
}

pub struct RawCompletionQueue {}

impl RawCompletionQueue {
    pub fn pop(&mut self) -> Option<RawCqe> {
        todo!()
    }
}

pub struct SubmissionQueue<C> {
    _command_set: PhantomData<C>,
}

pub struct CompletionQueue<C> {
    _command_set: PhantomData<C>,
}

pub struct RawSqe {}

pub struct RawCqe {}

pub struct IoQueue {}

pub struct IoSq {}

pub struct IoCq {}

#[repr(C)]
#[derive(Debug)]
struct Registers {
    attrs: ControllerAttributes,
    _pad: [u8; 0x1000 - mem::size_of::<ControllerAttributes>()],
    doorbells: [Volatile<u32>],
}
