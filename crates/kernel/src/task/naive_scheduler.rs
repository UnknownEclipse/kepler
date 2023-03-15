use core::{cell::SyncUnsafeCell, mem};

use hal::interrupts;
use meteor::tail_list::TailList;
use spin::{mutex::SpinMutex, Once};

mod queue;
mod soul;

use self::{queue::TaskQueue, soul::Soul};
use super::{sched::Scheduler, task_types::Task};
use crate::error::{KernErrorKind, KernResult};

/// A dumb as rocks single-soul scheduler.
pub struct NaiveScheduler {
    soul: Once<SyncUnsafeCell<Soul>>,
    queue: SpinMutex<TaskQueue>,
}

impl NaiveScheduler {
    pub fn new() -> Self {
        Self {
            soul: Once::new(),
            queue: Default::default(),
        }
    }

    fn with_soul<F, T>(&self, f: F) -> KernResult<T>
    where
        F: FnOnce(&mut Soul) -> KernResult<T>,
    {
        interrupts::without(|_| {
            let soul = self.soul.get().ok_or(KernErrorKind::Fault)?.get();
            unsafe { f(&mut *soul) }
        })
    }
}

impl Scheduler for NaiveScheduler {
    fn unpark(&self, task: Task) -> KernResult<()> {
        interrupts::without(|_| unsafe {
            if let Some(soul) = self.soul.get() {
                (*soul.get()).unpark(task);
            } else {
                self.queue.lock().push(task);
            }
        });
        Ok(())
    }

    fn park(&self) -> KernResult<()> {
        self.with_soul(|s| {
            s.park();
            Ok(())
        })
    }

    fn current(&self) -> KernResult<Task> {
        self.with_soul(|s| Ok(s.current()))
    }

    fn yield_now(&self) -> KernResult<()> {
        self.with_soul(|s| {
            s.yield_now();
            Ok(())
        })
    }

    fn exit(&self) -> KernResult<!> {
        self.with_soul(|s| {
            s.exit();
        })
    }

    unsafe fn enter(&self) -> KernResult<!> {
        let queue = mem::take(&mut *self.queue.lock());
        self.soul
            .call_once(|| SyncUnsafeCell::new(Soul::new(queue)));

        self.with_soul(|s| {
            s.enter();
        })
    }
}
