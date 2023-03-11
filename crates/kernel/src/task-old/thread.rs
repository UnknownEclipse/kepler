use alloc::{boxed::Box, string::String, sync::Arc};
use core::{
    cell::SyncUnsafeCell,
    fmt::{Debug, Display},
    mem::MaybeUninit,
    num::NonZeroU64,
    ptr::{self, NonNull},
    sync::atomic::{AtomicBool, AtomicPtr, AtomicU64, AtomicUsize, Ordering},
    task::Waker,
};

use hal::{
    interrupts,
    task::{context_switch, hw_thread_id, Context},
};
use log::trace;

use super::scheduler::Scheduler;
use crate::{
    error::KernResult,
    memory::{AddrSpace, AllocOptions},
    task::scheduler,
};

pub fn park() {
    interrupts::without(|_| unsafe {
        scheduler().redispatch();
    })
}

pub fn park_or_wait() {
    unsafe {
        let were_enabled = interrupts::are_enabled();
        if were_enabled {
            interrupts::disable();
        }
        if !scheduler().redispatch() {
            trace!("cpu {} waiting for interrupts", hw_thread_id());
            interrupts::enable_and_wait();
        } else if were_enabled {
            interrupts::enable();
        }
    }
}

pub fn yield_now() {
    interrupts::without(|_| unsafe {
        scheduler().yield_now();
    })
}

pub fn current() -> Thread {
    interrupts::without(|_| unsafe { scheduler().current() })
}

pub unsafe fn has_waiting_threads() -> bool {
    debug_assert!(!interrupts::are_enabled());
    scheduler().has_waiting_threads()
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

    pub unsafe fn from_raw(raw: NonNull<()>) -> Self {
        Self(Arc::from_raw(raw.as_ptr().cast()))
    }

    pub fn into_raw(self) -> NonNull<()> {
        NonNull::new(Arc::into_raw(self.0).cast_mut())
            .expect("ptr will never be null")
            .cast()
    }

    /// Convert this thread into a [`Waker`]. This will never allocate, and behind
    /// the scenes calls `unpark()` and `unpark_ref()`.
    ///
    /// This can be useful when working with wait lists that combine multiple types
    /// of objects (threads, futures, etc.)
    pub fn into_waker(self) -> Waker {
        let raw = waker::from_thread(self);
        unsafe { Waker::from_raw(raw) }
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
        self.try_schedule_onto(scheduler()).ok();
    }

    pub fn unpark_ref(&self) {
        self.clone().unpark();
    }

    pub(super) fn try_schedule_onto<S>(self, scheduler: &S) -> Result<(), Thread>
    where
        S: ?Sized + Scheduler,
    {
        if self.try_schedule() {
            unsafe { scheduler.schedule(self) };
            Ok(())
        } else {
            Err(self)
        }
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
        debug_assert!(
            self.0.is_scheduled.swap(false, Ordering::Release),
            "thread was not scheduled"
        );
    }
}

impl Display for Thread {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(name) = self.name() {
            write!(f, "<{}>", name)
        } else {
            write!(f, "{:#x}", self.id().0)
        }
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

    pub fn spawn_raw(self, start: StartFn, data: *mut ()) -> KernResult<Thread> {
        let mut stack = allocate_stack(self.stack_size, &self.addr_space)?;
        let saved_context = init_stack(&mut stack, start, data);

        let inner = Inner {
            affinity: AtomicUsize::new(0),
            id: allocate_id(),
            is_scheduled: AtomicBool::new(false),
            name: self.name,
            saved_context: AtomicPtr::new(saved_context),
            stack,
            addr_space: self.addr_space,
        };
        let thread = Thread(Arc::new(inner));
        thread.unpark_ref();
        Ok(thread)
    }

    pub fn spawn<F>(self, f: F) -> KernResult<Thread>
    where
        F: FnOnce() + Send + 'static,
    {
        extern "C" fn start<F>(data: *mut ()) -> !
        where
            F: FnOnce() + Send + 'static,
        {
            let slot: *mut Option<F> = data.cast();
            let f = unsafe { (*slot).take().unwrap_unchecked() };
            f();

            loop {
                park_or_wait();
            }
        }

        let mut f = Some(f);
        let data: *mut Option<F> = &mut f;
        self.spawn_raw(start::<F>, data.cast())
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
    // TODO: Is it worth while to embed a linked list link to threads themselves?
    // Benefits:
    //   Safe chained scheduling. (T0 -> T1) can be enforced safely (with spurious
    //   wakeups possible)
}

impl Drop for Inner {
    fn drop(&mut self) {
        trace!("thread {:#x} dropped", self.id.0);
    }
}

fn allocate_id() -> ThreadId {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let val = COUNTER.fetch_add(1, Ordering::Relaxed);
    NonZeroU64::new(val)
        .map(ThreadId)
        .expect("thread id counter overflow")
}

pub(super) unsafe fn switch_threads(old: &Thread, new: &Thread) {
    let old = old.0.saved_context.as_ptr();
    let new = new.0.saved_context.load(Ordering::Acquire);
    context_switch(old, new);
}

fn allocate_stack(
    size: usize,
    addr_space: &AddrSpace,
) -> KernResult<Box<SyncUnsafeCell<[MaybeUninit<u8>]>>> {
    let region = AllocOptions::new(size)
        .start_guard_pages(1)
        .end_guard_pages(1)
        .allocate_in_address_space(addr_space)?;

    let ptr = region.as_ptr() as *mut _;
    unsafe { Ok(Box::from_raw(ptr)) }
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

mod waker {
    use core::{
        mem::ManuallyDrop,
        ops::Deref,
        ptr::NonNull,
        task::{RawWaker, RawWakerVTable},
    };

    use super::Thread;

    const RAW_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

    pub fn from_thread(thread: Thread) -> RawWaker {
        let data = thread.into_raw().as_ptr();
        let vtable = &RAW_WAKER_VTABLE;
        RawWaker::new(data, vtable)
    }

    unsafe fn get(ptr: *const ()) -> Thread {
        let raw = NonNull::new_unchecked(ptr.cast_mut());
        Thread::from_raw(raw)
    }

    unsafe fn clone(ptr: *const ()) -> RawWaker {
        from_thread(ManuallyDrop::new(get(ptr)).deref().clone())
    }

    unsafe fn wake(ptr: *const ()) {
        get(ptr).unpark();
    }

    unsafe fn wake_by_ref(ptr: *const ()) {
        ManuallyDrop::new(get(ptr)).unpark_ref();
    }

    unsafe fn drop(ptr: *const ()) {
        _ = get(ptr);
    }
}
