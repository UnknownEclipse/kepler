use alloc::boxed::Box;
use core::sync::atomic::Ordering;

use hal::task::context_switch;
use log::trace;
use spin::Once;

pub use self::task_types::{Task, TaskId};
use self::{naive_scheduler::NaiveScheduler, sched::Scheduler};
use crate::error::KernResult;

mod idle;
mod naive_scheduler;
mod sched;
mod stack;
mod task_types;
mod thread;

pub fn spawn<F, T>(f: F) -> KernResult<Task>
where
    F: FnOnce() -> T + 'static + Send,
{
    thread::spawn(f).map(|t| t.0)
}

pub fn yield_now() {
    scheduler().yield_now();
}

pub fn park() {
    trace!("scheduler.park: {:?}", current());
    scheduler().park();
}

pub fn current() -> Task {
    scheduler().current()
}

pub fn exit() -> ! {
    trace!("scheduler.exit: {:?}", current());
    scheduler().exit();
}

pub unsafe fn enter() -> ! {
    scheduler().enter();
}

fn scheduler() -> &'static dyn Scheduler {
    *SCHEDULER.get().expect("scheduler not initialized")
}

static SCHEDULER: Once<&'static dyn Scheduler> = Once::new();

#[derive(Debug)]
pub struct InitSchedulerError;

pub fn try_init_scheduler<S>(sched: &'static S) -> Result<(), InitSchedulerError>
where
    S: Scheduler,
{
    let mut init = false;
    SCHEDULER.call_once(|| {
        init = true;
        sched
    });
    if init {
        Ok(())
    } else {
        Err(InitSchedulerError)
    }
}

unsafe fn task_switch(old: &Task, new: &Task) {
    let old = old.head().stack_ptr.as_ptr();
    let new = new.head().stack_ptr.load(Ordering::Relaxed);
    context_switch(old, new);
}

pub fn naive() {
    let s = Box::new(NaiveScheduler::new());
    let s = Box::leak(s);
    try_init_scheduler(s).unwrap();
}
