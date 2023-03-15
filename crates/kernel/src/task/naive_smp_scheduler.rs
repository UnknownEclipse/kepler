use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::{
    cell::{Cell, SyncUnsafeCell},
    mem, ptr,
    sync::atomic::Ordering,
};

use hal::{
    interrupts::{self, WithoutInterrupts},
    task::{context_switch, hw_thread_id},
};
use log::{trace, warn};
use meteor::mpsc_queue::{Link, MpscQueue};
use nanorand::{Rng, WyRand};
use spin::{mutex::SpinMutex, Once};

use super::{sched::Scheduler, Task};
use crate::{
    arch::interrupts::enable_and_wait,
    error::{KernErrorKind, KernResult},
    task::{idle::allocate_bootstrap_task, task_types::State},
};

static STUB: Link = Link::new();

pub struct NaiveSmpScheduler {
    workers: Box<[Worker]>,
    queue: Arc<MpmcQueue>,
}

unsafe impl Sync for NaiveSmpScheduler {}

impl NaiveSmpScheduler {
    pub fn new(cores: usize) -> Self {
        let worker_count = cores;
        let mut workers = Vec::with_capacity(worker_count);

        let queue = Arc::new(MpmcQueue::new());

        let mut seed = 0;
        workers.resize_with(worker_count, || {
            seed += 1;
            Worker {
                current: Once::new(),
                exited: Cell::new(None),
                queue: queue.clone(),
            }
        });
        let workers = workers.into_boxed_slice();

        Self { workers, queue }
    }

    fn with_worker<F, T>(&self, f: F) -> KernResult<T>
    where
        F: FnOnce(&Worker) -> KernResult<T>,
    {
        interrupts::without(|guard| {
            let worker = self.worker(guard);
            f(worker)
        })
    }

    fn worker<'a>(&'a self, _g: &'a WithoutInterrupts) -> &'a Worker {
        unsafe { self.worker_unchecked() }
    }

    unsafe fn worker_unchecked(&self) -> &Worker {
        let cpu = hw_thread_id();
        &self.workers[cpu]
    }
}

impl Scheduler for NaiveSmpScheduler {
    fn unpark(&self, task: Task) -> KernResult<()> {
        self.queue.push(task);
        Ok(())
    }

    fn park(&self) -> KernResult<()> {
        self.with_worker(|worker| {
            let new = worker.queue.pop().expect("no tasks in queue");
            worker.switch(new, false);
            Ok(())
        })
    }

    fn current(&self) -> KernResult<Task> {
        self.with_worker(|worker| {
            let cell = worker.current.get().ok_or(KernErrorKind::Fault)?;
            let task = unsafe { (*cell.get()).clone() };
            Ok(task)
        })
    }

    fn yield_now(&self) -> KernResult<()> {
        self.with_worker(|worker| {
            let Some(new) = self.queue.pop() else { return Ok(()) };
            worker.switch(new, true);
            Ok(())
        })
    }

    fn exit(&self) -> KernResult<!> {
        self.with_worker(|worker| {
            let new = self.queue.pop().expect("no waiting tasks");
            worker.switch(new, false);
            unreachable!();
        })
    }

    unsafe fn enter(&self) -> KernResult<!> {
        self.with_worker(|worker| {
            worker
                .current
                .call_once(|| SyncUnsafeCell::new(allocate_bootstrap_task()));

            Ok(())
        })?;

        loop {
            interrupts::disable();

            if let Some(new) = self.queue.pop() {
                self.worker_unchecked().switch(new, true);
            } else {
                warn!("halt");
                unsafe { enable_and_wait() };
            }
        }
    }
}

struct Worker {
    current: Once<SyncUnsafeCell<Task>>,
    exited: Cell<Option<Task>>,
    queue: Arc<MpmcQueue>,
}

impl Worker {
    fn switch(&self, new: Task, requeue: bool) {
        let slot = self.current.get().expect("uninitialized worker");
        let old = unsafe { ptr::replace(slot.get(), new) };

        let active = unsafe { &*slot.get() };

        trace!("switch {} -> {}", old, active);

        active.change_state_to_active();

        let old_state = if requeue {
            self.queue.push(old.clone());
            State::Queued
        } else {
            State::Parked
        };

        old.change_state(State::Active, old_state)
            .expect("invalid task state transition");

        let old_ctx = old.head().stack_ptr.as_ptr();
        let new_ctx = active.saved_context();

        trace!("num_refs = {}", old.head().refs.load(Ordering::Relaxed));

        if requeue {
            mem::drop(old)
        } else {
            self.exited.set(Some(old));
        }

        unsafe {
            context_switch(old_ctx, new_ctx);
        }
    }
}

struct MpmcQueue {
    inner: MpscQueue<Task>,
    pop_lock: SpinMutex<()>,
}

impl MpmcQueue {
    pub fn new() -> Self {
        let stub = Box::new(Link::new());

        Self {
            inner: MpscQueue::with_static_stub(Box::leak(stub)),
            pop_lock: SpinMutex::new(()),
        }
    }

    pub fn pop(&self) -> Option<Task> {
        let _guard = self.pop_lock.lock();
        unsafe { self.inner.pop_unsync() }
    }

    pub fn try_pop(&self) -> Option<Task> {
        let _guard = self.pop_lock.try_lock()?;
        unsafe { self.inner.pop_unsync() }
    }

    pub fn push(&self, task: Task) {
        self.inner.push(task);
    }
}
