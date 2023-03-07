use core::{marker::PhantomData, sync::atomic::Ordering};

use super::raw_task::RawTask;
use crate::thread::Thread;

#[derive(Debug)]
pub struct JoinHandle<T> {
    thread: Thread,
    _p: PhantomData<fn(T)>,
}

impl<T> JoinHandle<T> {
    pub(crate) unsafe fn from_raw(raw: RawTask) -> Self {
        JoinHandle {
            thread: Thread(raw),
            _p: PhantomData,
        }
    }

    pub fn join(self) -> T {
        self.task().header().finished.wait();
        unsafe { self.task().take_value().expect("value not present") }
    }

    pub fn is_finished(&self) -> bool {
        self.task().header().finished.is_set()
    }

    pub fn thread(&self) -> &Thread {
        &self.thread
    }

    pub(crate) fn task(&self) -> &RawTask {
        &self.thread.0
    }
}

impl<T> Drop for JoinHandle<T> {
    fn drop(&mut self) {
        unsafe { self.task().detach() };
    }
}
