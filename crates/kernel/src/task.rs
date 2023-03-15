use alloc::boxed::Box;
use core::sync::atomic::Ordering;

use hal::task::context_switch;
use log::trace;
use spin::Once;

pub use self::task_types::{Task, TaskId};
use self::{
    naive_scheduler::NaiveScheduler, naive_smp_scheduler::NaiveSmpScheduler, sched::Scheduler,
    work_stealing::WorkStealingScheduler,
};
use crate::error::{KernErrorKind, KernResult};

mod idle;
mod naive_scheduler;
mod naive_smp_scheduler;
mod process;
mod sched;
mod stack;
mod task_types;
mod thread;
mod wait_list;
mod work_stealing;

#[derive(Debug)]
pub struct SchedError;

pub fn spawn<F, T>(f: F) -> KernResult<Task>
where
    F: FnOnce() -> T + 'static + Send,
{
    thread::spawn(f)
}

pub fn yield_now() {
    try_yield_now().unwrap();
}

pub fn park() {
    try_park().unwrap();
}

pub fn unpark(task: Task) {
    try_unpark(task).unwrap()
}
pub fn current() -> Task {
    try_current().unwrap()
}

pub fn exit() -> ! {
    try_exit().unwrap()
}

pub fn try_yield_now() -> KernResult<()> {
    scheduler()?.yield_now()
}

pub fn try_park() -> KernResult<()> {
    trace!("scheduler.park: {:?}", current());
    scheduler()?.park()
}

pub fn try_current() -> KernResult<Task> {
    scheduler()?.current()
}

pub fn try_exit() -> KernResult<!> {
    scheduler()?.exit()
}

pub fn try_unpark(task: Task) -> KernResult<()> {
    scheduler()?.unpark(task)
}

pub unsafe fn try_enter() -> KernResult<!> {
    scheduler()?.enter()
}

pub unsafe fn enter() -> ! {
    try_enter().unwrap()
}

fn scheduler() -> KernResult<&'static dyn Scheduler> {
    Ok(*SCHEDULER.get().ok_or(KernErrorKind::Fault)?)
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

pub fn init_naive_scheduler() {
    let s = Box::new(NaiveScheduler::new());
    let s = Box::leak(s);
    try_init_scheduler(s).unwrap();
}

pub fn init_smp_scheduler(cores: usize) {
    let s = Box::new(WorkStealingScheduler::new(cores));
    let s = Box::leak(s);
    try_init_scheduler(s).unwrap();
}

pub fn init_naive_smp_scheduler(cores: usize) {
    let s = Box::new(NaiveSmpScheduler::new(cores));
    let s = Box::leak(s);
    try_init_scheduler(s).unwrap();
}
