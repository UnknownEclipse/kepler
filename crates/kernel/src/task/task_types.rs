use core::{
    fmt::{Debug, Display},
    hint::unreachable_unchecked,
    mem::ManuallyDrop,
    num::NonZeroU64,
    ptr::NonNull,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicU64, AtomicU8, AtomicUsize, Ordering},
};

use hal::task::Context;
use log::info;
use meteor::{DynSinglePtrLink, Node};

use super::unpark;
use crate::memory::AddrSpace;

#[derive(Debug)]
pub struct Head {
    pub link: AtomicPtr<DynSinglePtrLink>,
    pub state: AtomicState,
    pub refs: AtomicUsize,
    pub id: TaskId,
    pub vtable: &'static TaskVTable,
    pub stack_ptr: AtomicPtr<Context>,
    pub policy: Policy,
    pub preemptible: AtomicBool,
}

impl Drop for Head {
    fn drop(&mut self) {
        info!("task dropped");
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Queued,
    Active,
    Parked,
    Exited,
}

impl State {
    unsafe fn from_u8_unchecked(v: u8) -> Self {
        match v {
            0 => State::Queued,
            1 => State::Active,
            2 => State::Parked,
            3 => State::Exited,
            _ => unreachable_unchecked(),
        }
    }
}

pub struct AtomicState(AtomicU8);

impl AtomicState {
    pub const fn new(state: State) -> Self {
        Self(AtomicU8::new(state as u8))
    }

    pub fn load(&self, order: Ordering) -> State {
        unsafe { State::from_u8_unchecked(self.0.load(order)) }
    }

    pub fn store(&self, value: State, order: Ordering) {
        self.0.store(value as u8, order);
    }

    pub fn compare_exchange(
        &self,
        current: State,
        new: State,
        success: Ordering,
        failure: Ordering,
    ) -> Result<State, State> {
        self.0
            .compare_exchange(current as u8, new as u8, success, failure)
            .map(|v| unsafe { State::from_u8_unchecked(v) })
            .map_err(|v| unsafe { State::from_u8_unchecked(v) })
    }

    pub fn compare_exchange_weak(
        &self,
        current: State,
        new: State,
        success: Ordering,
        failure: Ordering,
    ) -> Result<State, State> {
        self.0
            .compare_exchange_weak(current as u8, new as u8, success, failure)
            .map(|v| unsafe { State::from_u8_unchecked(v) })
            .map_err(|v| unsafe { State::from_u8_unchecked(v) })
    }

    pub fn swap(&self, value: State, order: Ordering) -> State {
        unsafe { State::from_u8_unchecked(self.0.swap(value as u8, order)) }
    }
}

impl Debug for AtomicState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("AtomicState")
            .field(&self.load(Ordering::Relaxed))
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Policy {
    Low(u8),
    Normal(u8),
    High(u8),
}

impl Policy {
    pub fn should_preempt(&self, policy: Policy) -> bool {
        match (self, policy) {
            (Policy::Low(_), _) => false,
            (this, other) => *this > other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(NonZeroU64);

pub struct Task(pub NonNull<Head>);

impl Task {
    pub unsafe fn from_raw(raw: NonNull<Head>) -> Self {
        Self(raw)
    }

    pub fn into_raw(self) -> NonNull<Head> {
        ManuallyDrop::new(self).0
    }

    pub fn unpark(self) {
        unpark(self);
    }

    pub fn id(&self) -> TaskId {
        self.head().id
    }

    pub fn name(&self) -> Option<&str> {
        None
    }

    pub fn head(&self) -> &Head {
        unsafe { self.0.as_ref() }
    }

    pub fn vtable(&self) -> &'static TaskVTable {
        self.head().vtable
    }

    pub fn change_state(&self, old: State, new: State) -> Result<(), State> {
        self.head()
            .state
            .compare_exchange(old, new, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ())
    }

    pub fn change_state_to_active(&self) {
        let old_state = self.head().state.swap(State::Active, Ordering::AcqRel);
        if !matches!(old_state, State::Queued | State::Parked) {
            panic!("invalid task state transition");
        }
    }

    pub fn saved_context(&self) -> *mut Context {
        self.head().stack_ptr.load(Ordering::Acquire)
    }

    pub fn address_space(&self) -> &AddrSpace {
        &AddrSpace::Kernel
    }

    pub fn policy(&self) -> Policy {
        self.head().policy
    }
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}

impl Clone for Task {
    fn clone(&self) -> Self {
        self.head().refs.fetch_add(1, Ordering::Relaxed);
        Self(self.0)
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        let n = self.head().refs.fetch_sub(1, Ordering::Relaxed);

        if 1 != n {
            info!("task.drop refs = {}", n);
            return;
        }

        let vtable = self.vtable();
        unsafe {
            (vtable.drop_in_place)(self.0);
            (vtable.deallocate)(self.0.cast());
        }
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(name) = self.name() {
            write!(f, "<{}>", name)
        } else {
            write!(f, "{:#x}", self.id().0)
        }
    }
}

impl Debug for Task {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Task").field("id", &self.head().id).finish()
    }
}

impl Node<DynSinglePtrLink> for Task {
    fn into_link(node: Self) -> NonNull<DynSinglePtrLink> {
        node.into_raw().cast()
    }

    unsafe fn from_link(link: NonNull<DynSinglePtrLink>) -> Self {
        Task::from_raw(link.cast())
    }
}

pub struct TaskVTable {
    pub drop_in_place: unsafe fn(NonNull<Head>),
    pub deallocate: unsafe fn(NonNull<u8>),
}

impl Debug for TaskVTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TaskVTable").finish_non_exhaustive()
    }
}

pub fn allocate_id() -> TaskId {
    static V: AtomicU64 = AtomicU64::new(0);
    NonZeroU64::new(V.fetch_add(1, Ordering::Relaxed) + 1)
        .map(TaskId)
        .expect("task id overflow")
}
