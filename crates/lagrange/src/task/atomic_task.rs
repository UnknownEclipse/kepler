use core::mem;

use spin::mutex::SpinMutex;

use super::raw_task::RawTask;

#[derive(Debug)]
pub struct AtomicTask {
    inner: SpinMutex<RawTask>,
}

impl AtomicTask {
    pub fn new(task: RawTask) -> Self {
        Self {
            inner: SpinMutex::new(task),
        }
    }

    pub fn load(&self) -> RawTask {
        self.inner.lock().clone()
    }

    pub fn store(&self, task: RawTask) {
        *self.inner.lock() = task;
    }

    pub fn swap(&self, task: RawTask) -> RawTask {
        mem::replace(&mut *self.inner.lock(), task)
    }
}
