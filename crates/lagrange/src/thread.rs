use core::num::NonZeroU64;

use crate::task::{raw_task::RawTask, JoinHandle};

pub fn spawn<F, R>(f: F) -> JoinHandle<()>
where
    F: Send + FnOnce() -> R,
    R: Send,
{
    todo!()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ThreadId(NonZeroU64);

#[derive(Debug, Clone)]
pub struct Thread {
    task: RawTask,
}

impl Thread {
    pub fn unpark(self) {
        self.task.schedule();
    }

    pub fn name(&self) -> Option<&str> {
        self.task.name()
    }

    pub fn id(&self) -> ThreadId {
        ThreadId(self.task.id())
    }
}

pub fn current() -> Thread {
    todo!()
}

pub fn park() {}
