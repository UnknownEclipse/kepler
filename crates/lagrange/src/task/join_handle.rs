use core::marker::PhantomData;

use super::raw_task::RawTask;

#[derive(Debug)]
pub struct JoinHandle<T> {
    raw: RawTask,
    _p: PhantomData<fn(T)>,
}

impl<T> JoinHandle<T> {
    pub(crate) unsafe fn from_raw(raw: RawTask) -> Self {
        JoinHandle {
            raw,
            _p: PhantomData,
        }
    }

    pub fn join(self) -> T {
        todo!("wait");
    }
}

impl<T> Drop for JoinHandle<T> {
    fn drop(&mut self) {
        unsafe { self.raw.detach() };
    }
}
