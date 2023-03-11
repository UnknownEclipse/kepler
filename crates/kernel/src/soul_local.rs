use alloc::{boxed::Box, vec::Vec};

use hal::{interrupts::WithoutInterrupts, task::hw_thread_id};
use spin::Once;

pub struct SoulLocal<T> {
    array: Box<[Once<T>]>,
}

impl<T> SoulLocal<T> {
    pub fn new() -> Self {
        let mut v = Vec::with_capacity(1);
        v.resize_with(1, Once::new);
        Self {
            array: v.into_boxed_slice(),
        }
    }

    pub fn get<'a>(&'a self, _token: &'a WithoutInterrupts) -> Option<&'a T> {
        let i = unsafe { hw_thread_id() };
        self.array.get(i).and_then(|v| v.get())
    }

    pub fn get_or_default<'a>(&'a self, _token: &'a WithoutInterrupts) -> &'a T
    where
        T: Default,
    {
        let i = unsafe { hw_thread_id() };
        self.array[i].call_once(|| Default::default())
    }

    pub fn get_or_init<'a, F>(&'a self, init: F, _token: &'a WithoutInterrupts) -> &'a T
    where
        F: FnOnce() -> T,
    {
        let i = unsafe { hw_thread_id() };
        self.array[i].call_once(|| init())
    }
}

unsafe impl<T> Sync for SoulLocal<T> where T: Send {}
