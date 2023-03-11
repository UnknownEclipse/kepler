use alloc::{
    boxed::Box,
    collections::VecDeque,
    sync::{Arc, Weak},
    vec::Vec,
};
use core::{cell::SyncUnsafeCell, cmp, ptr};

use hal::task::hw_thread_id;
use spin::mutex::SpinMutex;

use super::{scheduler::Scheduler, spmc::UnsafeQueue, thread::Thread};
use crate::task::thread::switch_threads;

pub struct NgScheduler {
    inner: Arc<NgSchedulerInner>,
}

impl NgScheduler {
    pub fn new(hw_threads: usize) -> Self {
        let inner = Arc::new_cyclic(|inner| {
            let mut threads = Vec::with_capacity(hw_threads);
            for i in 0..hw_threads {
                let thread = NgHwThread {
                    current: SyncUnsafeCell::new(None),
                    id: i,
                    inner: inner.clone(),
                    local: UnsafeQueue::new(),
                };
                threads.push(thread);
            }

            NgSchedulerInner {
                threads: threads.into_boxed_slice(),
                global: Default::default(),
            }
        });
        Self { inner }
    }
}
impl Scheduler for NgScheduler {
    unsafe fn enter(&self) {
        let hw_thread = hw_thread_id();
        self.inner.threads[hw_thread].enter();
    }

    unsafe fn schedule(&self, thread: Thread) {
        let current = hw_thread_id();
        self.inner.threads[current].schedule(thread);
    }

    unsafe fn yield_now(&self) {
        let hw_thread = hw_thread_id();
        self.inner.threads[hw_thread].yield_now();
    }

    unsafe fn redispatch(&self) -> bool {
        let hw_thread = hw_thread_id();
        self.inner.threads[hw_thread].redispatch()
    }

    unsafe fn current(&self) -> Thread {
        let hw_thread = hw_thread_id();
        self.inner.threads[hw_thread].current()
    }

    unsafe fn has_waiting_threads(&self) -> bool {
        true
    }
}

struct NgSchedulerInner {
    threads: Box<[NgHwThread]>,
    global: SpinMutex<VecDeque<Thread>>,
}

struct NgHwThread {
    local: UnsafeQueue,
    current: SyncUnsafeCell<Option<Thread>>,
    inner: Weak<NgSchedulerInner>,
    id: usize,
}

impl NgHwThread {
    unsafe fn enter(&self) {
        let this = Thread::current(None);
        *self.current.get() = Some(this);
    }

    unsafe fn current(&self) -> Thread {
        (*self.current.get())
            .clone()
            .expect("thread not initialized")
    }

    unsafe fn schedule(&self, task: Thread) {
        if let Err(thread) = self.local.push(task) {
            self.inner().global.lock().push_back(thread);
        }
    }

    unsafe fn yield_now(&self) {
        if let Some(new) = self.next() {
            let old = self.current();
            new.deschedule();
            old.unpark();
            self.switch_to(new);
        }
    }

    unsafe fn redispatch(&self) -> bool {
        if let Some(new) = self.next() {
            new.deschedule();
            self.switch_to(new);
            true
        } else {
            false
        }
    }

    unsafe fn switch_to(&self, new: Thread) {
        let cur = self.current.get();
        let old = ptr::replace(cur, Some(new.clone())).expect("executor not initialized");
        switch_threads(&old, &new);
    }

    unsafe fn next(&self) -> Option<Thread> {
        self.local.pop().or_else(|| self.steal())
    }

    unsafe fn steal(&self) -> Option<Thread> {
        let inner = self.inner();
        for (i, thread) in inner.threads.iter().enumerate() {
            if i == self.id {
                continue;
            }
            if let Some(t) = thread.local.steal_into(&self.local) {
                return Some(t);
            }
        }

        let mut global = inner.global.lock();
        let n = cmp::min(global.len(), 64);
        if n == 0 {
            return None;
        }
        let t = global.pop_front();
        for _ in 1..n {
            if let Some(t) = global.pop_front() {
                self.local.push(t).expect("should not happen");
            }
        }
        t
    }

    fn inner(&self) -> Arc<NgSchedulerInner> {
        self.inner.upgrade().unwrap()
    }
}
