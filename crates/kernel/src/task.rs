use spin::Lazy;

use self::scheduler::Scheduler;

pub mod futex;
mod scheduler;
pub mod thread;

fn scheduler() -> &'static Scheduler {
    &GLOBAL
}

static GLOBAL: Lazy<Scheduler> = Lazy::new(Scheduler::new);

fn hw_thread_id() -> usize {
    0
}

pub unsafe fn enter(hw_id: usize) {
    scheduler().enter(hw_id);
}
