use alloc::{boxed::Box, collections::VecDeque, format, sync::Arc, vec::Vec};
use core::mem;

use hal::{interrupts, task::hw_thread_id};
use log::trace;
use spin::{mutex::SpinMutex, Once};

use super::{scheduler::Scheduler, thread::Thread};
use crate::task::thread::switch_threads;

#[derive(Debug)]
pub struct SimpleScheduler {
    threads: Box<[HwThread]>,
    queue: Arc<SpinMutex<VecDeque<Thread>>>,
}

impl SimpleScheduler {
    pub fn new(num_threads: usize) -> Self {
        let mut threads = Vec::new();
        let queue = Arc::new(SpinMutex::new(VecDeque::new()));
        threads.resize_with(num_threads, || HwThread {
            current: Once::new(),
            queue: queue.clone(),
        });
        let threads = threads.into_boxed_slice();

        Self { threads, queue }
    }

    fn thread(&self) -> &HwThread {
        let id = unsafe { hw_thread_id() };
        &self.threads[id]
    }
}

impl Scheduler for SimpleScheduler {
    unsafe fn enter(&self) {
        self.thread().enter();
    }

    unsafe fn schedule(&self, thread: Thread) {
        self.queue.lock().push_back(thread);
    }

    unsafe fn yield_now(&self) {
        self.thread().yield_now();
    }

    unsafe fn redispatch(&self) -> bool {
        self.thread().redispatch()
    }

    unsafe fn current(&self) -> Thread {
        self.thread().current()
    }

    unsafe fn has_waiting_threads(&self) -> bool {
        self.thread().has_waiting_threads()
    }
}

#[derive(Debug)]
struct HwThread {
    current: Once<SpinMutex<Thread>>,
    queue: Arc<SpinMutex<VecDeque<Thread>>>,
}

impl Scheduler for HwThread {
    unsafe fn enter(&self) {
        self.current
            .call_once(|| SpinMutex::new(Thread::current(None)));
    }

    unsafe fn schedule(&self, thread: Thread) {
        self.queue.lock().push_back(thread);
    }

    unsafe fn yield_now(&self) {
        let new = self.queue.lock().pop_front();
        if let Some(new) = new {
            new.deschedule();

            let mut cur = self.current.get().expect("thread not initialized").lock();

            let old = mem::replace(&mut *cur, new.clone());
            mem::drop(cur);

            old.clone().try_schedule_onto(self).ok();
            switch_threads(&old, &new);
        }
    }

    unsafe fn redispatch(&self) -> bool {
        let new = self.queue.lock().pop_front();
        if let Some(new) = new {
            new.deschedule();

            let mut cur = self.current.get().expect("thread not initialized").lock();
            let old = mem::replace(&mut *cur, new.clone());
            mem::drop(cur);

            switch_threads(&old, &new);
            true
        } else {
            false
        }
    }

    unsafe fn current(&self) -> Thread {
        self.current
            .get()
            .expect("thread not initialized")
            .lock()
            .clone()
    }

    unsafe fn has_waiting_threads(&self) -> bool {
        !self.queue.lock().is_empty()
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
