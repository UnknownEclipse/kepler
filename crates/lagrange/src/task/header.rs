use core::{
    fmt::Debug,
    ptr::NonNull,
    sync::atomic::{AtomicU32, AtomicUsize, Ordering},
};

use skua::mpsc_queue;

#[derive(Debug)]
pub struct Header {
    pub(super) mpsc_link: mpsc_queue::Link,
    pub(super) vtable: &'static TaskVTable,
    pub(super) state: State,
    pub(super) refs: AtomicUsize,
}

impl Header {
    // pub const fn new() -> Self {
    //     Self { mpsc_link: mpsc_queue::Link::new(), vtable: (), state: () }
    // }
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

pub(super) struct TaskVTable {
    pub drop: unsafe fn(NonNull<Header>),
    pub value_offset: usize,
    pub trailer_offset: usize,
}

impl Debug for TaskVTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TaskVTable").finish_non_exhaustive()
    }
}
