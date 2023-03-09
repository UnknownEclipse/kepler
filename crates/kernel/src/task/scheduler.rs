use alloc::{boxed::Box, collections::VecDeque, format};
use core::mem;

use hal::interrupts;
use log::trace;
use spin::{mutex::SpinMutex, Once};

use super::thread::Thread;
use crate::task::thread::switch_threads;

#[derive(Debug)]
pub struct Scheduler {
    execution_contexts: Box<[Once<ExecutionContext>]>,
}

impl Scheduler {
    pub fn new() -> Self {
        let execution_contexts = Box::new([Once::new()]);
        Self { execution_contexts }
    }

    pub unsafe fn enter(&self, hwt: usize) {
        trace!("beginning initialization for hw thread {}", hwt);
        let name = format!("<main {}>", hwt);
        let current = unsafe { Thread::current(Some(name)) };
        self.execution_contexts[hwt].call_once(|| ExecutionContext::new(current));
        trace!("finished initialization for hw thread {}", hwt);
    }

    pub(super) unsafe fn schedule(&self, thread: Thread) {
        debug_assert!(!interrupts::are_enabled());

        self.ctx(thread.affinity()).schedule(thread);
    }

    pub(super) unsafe fn current(&self, hwt: usize) -> Thread {
        self.ctx(hwt).current.lock().clone()
    }

    pub(super) unsafe fn yield_now(&self, hwt: usize) {
        self.ctx(hwt).yield_now();
    }

    pub(super) unsafe fn redispatch(&self, hwt: usize) {
        debug_assert!(!interrupts::are_enabled());

        self.ctx(hwt).redispatch();
    }

    fn ctx(&self, hwt: usize) -> &ExecutionContext {
        self.execution_contexts[hwt]
            .get()
            .expect("execution context not initialized")
    }
}

#[derive(Debug)]
struct ExecutionContext {
    current: SpinMutex<Thread>,
    queue: SpinMutex<VecDeque<Thread>>,
}

impl ExecutionContext {
    unsafe fn pop(&self) -> Option<Thread> {
        self.queue.lock().pop_front()
    }
}

impl ExecutionContext {
    pub fn new(current: Thread) -> Self {
        ExecutionContext {
            current: SpinMutex::new(current),
            queue: SpinMutex::new(VecDeque::new()),
        }
    }

    fn current(&self) -> Thread {
        self.current.lock().clone()
    }

    unsafe fn schedule(&self, task: Thread) {
        self.queue.lock().push_back(task);
    }

    unsafe fn yield_now(&self) {
        if let Some(new) = self.pop() {
            let old = self.current();
            new.deschedule();
            old.unpark();
            self.switch_to(new);
        }
    }

    unsafe fn redispatch(&self) {
        if let Some(new) = self.pop() {
            new.deschedule();
            self.switch_to(new);
        }
    }

    unsafe fn switch_to(&self, new: Thread) {
        let old = mem::replace(&mut *self.current.lock(), new.clone());
        switch_threads(&old, &new);
    }
}
