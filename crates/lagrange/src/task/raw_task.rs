use core::{mem::ManuallyDrop, num::NonZeroU64, ptr::NonNull};

use super::header::{Header, TaskVTable};
use crate::scheduler::Scheduler;

#[derive(Debug)]
pub struct RawTask(NonNull<Header>);

impl RawTask {
    pub unsafe fn detach(&self) {
        todo!()
    }

    pub fn id(&self) -> NonZeroU64 {
        todo!()
    }

    pub unsafe fn from_raw(raw: NonNull<()>) -> Self {
        Self(raw.cast())
    }

    pub fn into_raw(self) -> NonNull<()> {
        ManuallyDrop::new(self).0.cast()
    }

    fn value_ptr(&self) -> NonNull<()> {
        let ptr: *mut u8 = self.0.as_ptr().cast();
        let offset = self.vtable().value_offset;
        unsafe { NonNull::new_unchecked(ptr.add(offset).cast()) }
    }

    pub unsafe fn take_value<T>(&self) -> T {
        todo!()
    }

    fn header(&self) -> &Header {
        unsafe { self.0.as_ref() }
    }

    fn vtable(&self) -> &'static TaskVTable {
        self.header().vtable
    }

    pub fn name(&self) -> Option<&str> {
        None
    }

    pub fn schedule(self) {
        let s: *const Scheduler = self.scheduler();
        unsafe {
            (*s).schedule(self);
        }
    }

    pub fn scheduler(&self) -> &Scheduler {
        todo!()
    }
}

impl Clone for RawTask {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Drop for RawTask {
    fn drop(&mut self) {
        todo!()
    }
}
