use core::{
    fmt::Debug,
    mem::{ManuallyDrop, MaybeUninit},
    num::NonZeroU64,
    ptr::NonNull,
    sync::atomic::Ordering,
    task::Waker,
};

use super::{
    header::{Header, TaskVTable},
    waker::waker_from_raw_task,
};
use crate::scheduler::Scheduler;

const MAX_REFCOUNT: usize = isize::MAX as usize;

pub struct RawTask(NonNull<Header>);

impl RawTask {
    pub unsafe fn detach(&self) {
        // TODO: Proper detachment support
        //       I actually don't know what this would imply. We could avoid writing
        //       the return value, but I doubt that would have a significant performance
        //       impact.
    }

    pub fn id(&self) -> NonZeroU64 {
        self.header().id
    }

    pub unsafe fn from_raw(raw: NonNull<Header>) -> Self {
        Self(raw)
    }

    pub fn into_raw(self) -> NonNull<Header> {
        ManuallyDrop::new(self).0
    }

    pub unsafe fn take_value<T>(&self) -> Option<T> {
        if self.header().finished.is_set() {
            let f = self.vtable().read_value_into;
            let mut buf = MaybeUninit::<T>::uninit();
            (f)(self.0, buf.as_mut_ptr().cast());
            Some(buf.assume_init())
        } else {
            None
        }
    }

    pub fn header(&self) -> &Header {
        unsafe { self.0.as_ref() }
    }

    fn vtable(&self) -> &'static TaskVTable {
        self.header().vtable
    }

    pub fn name(&self) -> Option<&str> {
        self.header().name.as_deref()
    }

    pub fn schedule(self) {
        if let Some(s) = self.scheduler() {
            s.schedule(self);
        }
    }

    pub fn scheduler(&self) -> Option<&'static Scheduler> {
        self.header().scheduler.get()
    }

    /// Mark this task as scheduled, returning false if it is already scheduled elsewhere.
    pub fn set_scheduled(&self) -> bool {
        !self
            .header()
            .is_currently_scheduled
            .swap(true, Ordering::AcqRel)
    }

    pub fn mark_not_scheduled(&self) {
        self.header()
            .is_currently_scheduled
            .store(false, Ordering::Release);
    }

    pub fn into_waker(self) -> Waker {
        waker_from_raw_task(self)
    }
}

unsafe impl Send for RawTask {}
unsafe impl Sync for RawTask {}

impl Clone for RawTask {
    fn clone(&self) -> Self {
        unsafe {
            let nrefs = self.0.as_ref().refs.fetch_add(1, Ordering::Relaxed);
            if MAX_REFCOUNT < nrefs {
                panic!("refcount overflow");
            }
        }
        Self(self.0)
    }
}

impl Drop for RawTask {
    fn drop(&mut self) {
        unsafe {
            let nrefs = self.0.as_ref().refs.fetch_sub(1, Ordering::Relaxed);
            if 1 != nrefs {
                return;
            }

            let vtable = self.vtable();
            (vtable.drop_in_place)(self.0);
            (vtable.deallocate)(self.0.as_ptr().cast());
        }
    }
}

impl Debug for RawTask {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut s = f.debug_struct("RawTask");
        s.field("id", &self.id());
        if let Some(name) = self.name() {
            s.field("name", &name);
        }
        s.finish_non_exhaustive()
    }
}
