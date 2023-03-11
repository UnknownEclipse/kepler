use super::thread::Thread;

pub trait Scheduler {
    unsafe fn enter(&self);
    unsafe fn schedule(&self, thread: Thread);
    unsafe fn yield_now(&self);
    unsafe fn redispatch(&self) -> bool;
    unsafe fn current(&self) -> Thread;
    unsafe fn has_waiting_threads(&self) -> bool;
}
