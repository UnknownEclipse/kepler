use alloc::boxed::Box;
use core::{
    cell::Cell,
    mem::{self, ManuallyDrop},
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, Ordering},
};

use hal::{
    interrupts::{self, WithoutInterrupts},
    task::hw_thread_id,
};
use log::trace;
use spin::Once;

use crate::{
    arch::cpu::CpuId,
    error::{KernErrorKind, KernResult},
    memory::{AddrSpace, AllocOptions},
};

extern "C" {
    static __percpu_start: [u8; 4];
    static __percpu_stop: [u8; 4];
}

pub struct CpuLocal<T>
where
    T: Send,
{
    objects: Box<[Once<T>]>,
}

impl<T> CpuLocal<T>
where
    T: Send,
{
    pub fn get<'a>(&'a self, _guard: &'a WithoutInterrupts) -> Option<&'a T> {
        todo!()
    }

    pub fn get_or_try_init<'a, F, E>(
        &'a self,
        init: F,
        _guard: &'a WithoutInterrupts,
    ) -> Result<&'a T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        todo!()
    }

    pub fn get_or_init<'a, F>(&'a self, init: F, _guard: &'a WithoutInterrupts) -> &'a T
    where
        F: FnOnce() -> T,
    {
        todo!()
    }
}

unsafe impl<T> Sync for CpuLocal<T> where T: Send {}

static SECTIONS: [AtomicPtr<u8>; 64] = unsafe { mem::transmute([ptr::null_mut::<u8>(); 64]) };

fn percpu_section_size() -> usize {
    unsafe { __percpu_stop.as_ptr_range().end as usize - __percpu_start.as_ptr() as usize }
}

unsafe fn get_percpu_ptr<T>(ptr: NonNull<T>, cpu: usize) -> KernResult<NonNull<T>> {
    let offset = ptr.as_ptr() as usize - __percpu_start.as_ptr() as usize;
    let section = get_cpu_section(cpu)?;
    Ok(NonNull::new_unchecked(section.as_ptr().add(offset).cast()))
}

fn get_cpu_section(cpu: usize) -> KernResult<NonNull<u8>> {
    let atomic = SECTIONS.get(cpu).ok_or(KernErrorKind::Fault)?;
    let ptr = atomic.load(Ordering::Acquire);
    if let Some(ptr) = NonNull::new(ptr) {
        Ok(ptr)
    } else {
        trace!("allocating cpu local section for {:?}", CpuId::get());

        let pages = AllocOptions::new(percpu_section_size())
            .allocate_in_address_space(&AddrSpace::Kernel)?;

        unsafe {
            ptr::copy_nonoverlapping(
                __percpu_start.as_ptr(),
                pages.as_mut_ptr(),
                percpu_section_size(),
            )
        };

        atomic.store(pages.as_mut_ptr(), Ordering::Release);
        Ok(pages.as_non_null_ptr())
    }
}

macro_rules! cpu_local {
    ($vis:vis static $name:ident : $t:ty = $init:expr;) => {
        $vis static $name : $crate::cpu_local::LocalKey<$t> = unsafe { $crate::cpu_local::LocalKey::__new($init) };
    };
}

#[repr(transparent)]
#[derive(Debug)]
pub struct LocalKey<T>(ManuallyDrop<T>);

impl<T> LocalKey<T> {
    #[doc(hidden)]
    pub const unsafe fn __new(value: T) -> Self {
        Self(ManuallyDrop::new(value))
    }

    pub fn try_get<'a>(&'a self, _guard: &'a WithoutInterrupts) -> KernResult<&'a T> {
        unsafe { self.get_raw().map(|ptr| ptr.as_ref()) }
    }

    pub unsafe fn get_raw(&self) -> KernResult<NonNull<T>> {
        let cpu = hw_thread_id();
        let init = NonNull::from(self).cast::<T>();
        get_percpu_ptr(init, cpu)
    }

    pub fn with<F, R>(&self, f: F) -> KernResult<R>
    where
        F: FnOnce(&T) -> R,
    {
        interrupts::without(|guard| self.try_get(guard).map(f))
    }
}

unsafe impl<T> Sync for LocalKey<T> where T: Send {}

cpu_local! {
    static COUNTER: Cell<usize> = Cell::new(0);
}

fn use_percpu() {
    interrupts::without(|guard| {
        let value = COUNTER.try_get(guard).unwrap();
    });
}
