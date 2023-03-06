use core::{
    mem,
    ptr::NonNull,
    task::{RawWaker, RawWakerVTable, Waker},
};

use super::raw_task::RawTask;

pub fn waker_from_raw_task(task: RawTask) -> Waker {
    let data = into_raw(task);
    let vtable = &VTABLE;
    let raw_waker = RawWaker::new(data, vtable);
    unsafe { Waker::from_raw(raw_waker) }
}

const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

unsafe fn clone(ptr: *const ()) -> RawWaker {
    let task = from_raw(ptr);
    let new = task.clone();
    mem::forget(task);

    let data = into_raw(new);
    let vtable = &VTABLE;

    RawWaker::new(data, vtable)
}

unsafe fn wake(ptr: *const ()) {
    from_raw(ptr).schedule();
}

unsafe fn wake_by_ref(ptr: *const ()) {
    let task = from_raw(ptr);
    let new = task.clone();
    mem::forget(task);
    new.schedule();
}

unsafe fn drop(ptr: *const ()) {
    _ = from_raw(ptr);
}

unsafe fn from_raw(ptr: *const ()) -> RawTask {
    let raw = NonNull::new_unchecked(ptr.cast_mut()).cast();
    RawTask::from_raw(raw)
}

fn into_raw(task: RawTask) -> *const () {
    task.into_raw().as_ptr().cast()
}
