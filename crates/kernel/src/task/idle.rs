use alloc::boxed::Box;
use core::{
    ptr::NonNull,
    sync::atomic::{AtomicBool, AtomicUsize},
};

use super::task_types::{allocate_id, AtomicState, Head, Policy, State, Task, TaskVTable};

/// Create a task that refers to the current task.
pub unsafe fn allocate_bootstrap_task() -> Task {
    let head = Head {
        id: allocate_id(),
        link: Default::default(),
        policy: Policy::Low(0),
        preemptible: AtomicBool::new(true),
        refs: AtomicUsize::new(1),
        stack_ptr: Default::default(),
        state: AtomicState::new(State::Active),
        vtable: &VTABLE,
    };

    NonNull::new(Box::into_raw(Box::new(head)))
        .map(Task)
        .unwrap()
}

const VTABLE: TaskVTable = TaskVTable {
    deallocate: |head| unsafe {
        _ = Box::from_raw(head.as_ptr());
    },
    drop_in_place: |_head| {},
};
