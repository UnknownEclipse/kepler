use alloc::{borrow::Cow, boxed::Box};
use core::{
    cell::Cell,
    fmt::Debug,
    num::NonZeroU64,
    ptr::{self, NonNull},
    sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, AtomicU64, AtomicUsize, Ordering},
};

use hal::task::Context;
use skua::{mpsc_queue, Node};

use crate::{scheduler::Scheduler, sync::event::OneShotEvent};

#[repr(C)]
#[derive(Debug)]
pub struct Header {
    pub mpsc_link: mpsc_queue::Link,
    pub vtable: &'static TaskVTable,
    pub refs: AtomicUsize,
    pub is_currently_scheduled: AtomicBool,
    pub finished: OneShotEvent,
    pub context: AtomicPtr<Context>,
    pub scheduler: Cell<Option<&'static Scheduler>>,
    pub name: Option<Cow<'static, str>>,
    pub id: NonZeroU64,
    pub interrupts_enabled: AtomicBool,
}

impl Header {
    pub fn new(vtable: &'static TaskVTable) -> Self {
        Self {
            mpsc_link: mpsc_queue::Link::new(),
            vtable,
            refs: AtomicUsize::new(1),
            is_currently_scheduled: AtomicBool::new(false),
            finished: OneShotEvent::new(),
            context: AtomicPtr::new(ptr::null_mut()),
            scheduler: Cell::new(None),
            name: None,
            id: new_id(),
            interrupts_enabled: AtomicBool::new(true),
        }
    }
}

impl Node<mpsc_queue::Link> for Header {
    unsafe fn to_link(node: NonNull<Self>) -> NonNull<mpsc_queue::Link> {
        node.cast()
    }

    unsafe fn from_link(link: NonNull<mpsc_queue::Link>) -> NonNull<Self> {
        link.cast()
    }
}

#[derive(Debug)]
pub(super) struct State(AtomicU32);

impl State {
    pub fn inc_ref(&self) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }

    pub fn dec_ref(&self) -> u32 {
        (self.0.fetch_sub(1, Ordering::Relaxed) - 1) & !(0b11 << 30)
    }

    pub fn is_value_present(&self) -> bool {
        self.0.load(Ordering::Acquire) & (1 << 30) != 0
    }

    pub fn is_detached(&self) -> bool {
        self.0.load(Ordering::Acquire) & (1 << 31) != 0
    }
}

pub struct TaskVTable {
    pub drop_in_place: unsafe fn(NonNull<Header>),
    pub deallocate: unsafe fn(*mut u8),
    pub read_value_into: unsafe fn(NonNull<Header>, *mut u8),
}

impl Debug for TaskVTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TaskVTable").finish_non_exhaustive()
    }
}

pub fn new_id() -> NonZeroU64 {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    NonZeroU64::new(COUNTER.fetch_add(1, Ordering::Relaxed)).unwrap()
}
