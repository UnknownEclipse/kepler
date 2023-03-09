use alloc::{boxed::Box, string::String, sync::Arc};
use core::{
    cell::SyncUnsafeCell,
    fmt::Debug,
    iter::Step,
    mem::MaybeUninit,
    num::NonZeroU64,
    ptr,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicU64, AtomicUsize, Ordering},
};

use hal::{
    interrupts,
    task::{context_switch, Context},
};

use super::hw_thread_id;
use crate::{
    memory::{AddrSpace, KERNEL_ADDRESS_SPACE},
    task::scheduler,
};

pub fn park() {
    interrupts::without(|_| {
        let hwt = hw_thread_id();
        unsafe { scheduler().redispatch(hwt) };
    })
}

pub fn yield_now() {
    interrupts::without(|_| {
        let hwt = hw_thread_id();
        unsafe { scheduler().yield_now(hwt) };
    })
}

pub fn current() -> Thread {
    interrupts::without(|_| {
        let hwt = hw_thread_id();
        unsafe { scheduler().current(hwt) }
    })
}

pub type StartFn = extern "C" fn(*mut ()) -> !;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThreadId(NonZeroU64);

#[derive(Clone)]
pub struct Thread(Arc<Inner>);

impl Thread {
    pub(super) unsafe fn current(name: Option<String>) -> Thread {
        let inner = Inner {
            id: allocate_id(),
            name,
            saved_context: AtomicPtr::new(ptr::null_mut()),
            stack: Box::new(SyncUnsafeCell::new([])),
            affinity: AtomicUsize::new(hw_thread_id()),
            is_scheduled: AtomicBool::new(false),
            addr_space: AddrSpace::Kernel,
        };
        Thread(Arc::new(inner))
    }

    pub fn id(&self) -> ThreadId {
        self.0.id
    }

    pub fn name(&self) -> Option<&str> {
        self.0.name.as_deref()
    }

    pub fn affinity(&self) -> usize {
        self.0.affinity.load(Ordering::Acquire)
    }

    pub fn unpark(self) {
        _ = self.schedule();
    }

    pub fn unpark_ref(&self) {
        self.clone().unpark();
    }

    fn schedule(self) -> Result<(), Thread> {
        if !self.try_schedule() {
            return Err(self);
        }
        unsafe { scheduler().schedule(self) };
        Ok(())
    }

    fn try_schedule(&self) -> bool {
        !self.0.is_scheduled.swap(true, Ordering::AcqRel)
    }

    pub(super) fn deschedule(&self) {
        self.0.is_scheduled.store(false, Ordering::Release);
    }
}

impl Debug for Thread {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Thread")
            .field("id", &self.id())
            .field("name", &self.name())
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct Builder {
    stack_size: usize,
    name: Option<String>,
    addr_space: AddrSpace,
}

impl Builder {
    pub fn new(addr_space: AddrSpace) -> Self {
        Self {
            stack_size: 8192,
            name: None,
            addr_space,
        }
    }

    pub fn stack_size(mut self, stack_size: usize) -> Self {
        self.stack_size = stack_size;
        self
    }

    pub fn spawn(self, start: StartFn, data: *mut ()) -> Thread {
        let mut stack = allocate_stack(self.stack_size, &self.addr_space);
        let saved_context = init_stack(&mut stack, start, data);

        let inner = Inner {
            affinity: AtomicUsize::new(0),
            id: allocate_id(),
            is_scheduled: AtomicBool::new(true),
            name: self.name,
            saved_context: AtomicPtr::new(saved_context),
            stack,
            addr_space: self.addr_space,
        };
        let thread = Thread(Arc::new(inner));
        interrupts::without(|_| unsafe { scheduler().schedule(thread.clone()) });
        thread
    }
}

struct Inner {
    id: ThreadId,
    stack: Box<SyncUnsafeCell<[MaybeUninit<u8>]>>,
    name: Option<String>,
    saved_context: AtomicPtr<Context>,
    affinity: AtomicUsize,
    is_scheduled: AtomicBool,
    addr_space: AddrSpace,
}

fn allocate_id() -> ThreadId {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let val = COUNTER.fetch_add(1, Ordering::Relaxed);
    NonZeroU64::new(val)
        .map(ThreadId)
        .expect("thread id counter overflow")
}

pub(super) unsafe fn switch_threads(old: &Thread, new: &Thread) {
    let old = old.0.saved_context.as_mut_ptr();
    let new = new.0.saved_context.load(Ordering::Acquire);
    context_switch(old, new);
}

fn allocate_stack(size: usize, addr_space: &AddrSpace) -> Box<SyncUnsafeCell<[MaybeUninit<u8>]>> {
    let region =
        interrupts::without(|_| KERNEL_ADDRESS_SPACE.lock().allocate_virtual_region(size, 1))
            .unwrap();

    let ptr: *mut u8 = region.start.addr().as_ptr();
    let len = Step::steps_between(&region.start, &region.end).unwrap() * 4096;
    // let layout = Layout::from_size_align(size, 16).expect("invalid thread stack layout");

    let ptr = ptr::slice_from_raw_parts_mut(ptr, len);
    let ptr = ptr as *mut SyncUnsafeCell<[MaybeUninit<u8>]>;
    unsafe { Box::from_raw(ptr) }
}

fn init_stack(
    stack: &mut Box<SyncUnsafeCell<[MaybeUninit<u8>]>>,
    start: StartFn,
    data: *mut (),
) -> *mut Context {
    let top = stack.get_mut().as_mut_ptr_range().end;
    let top: *mut Context = top.cast();
    let initial_context = Context::with_initial(start, data);

    unsafe {
        let ctx = top.sub(1);
        ctx.write(initial_context);
        ctx
    }
}
