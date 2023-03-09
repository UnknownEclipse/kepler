#![no_std]

use alloc::vec::Vec;

use hal::{interrupts::NoInterrupts, intrin::hw_thread_id};

extern crate alloc;

pub struct HwThreadLocal<T> {
    items: Vec<T>,
}

impl<T> HwThreadLocal<T> {}

impl<T> HwThreadLocal<T> {
    pub fn get<'a>(&'a self, token: &'a NoInterrupts) -> &'a T {
        let id = unsafe { hw_thread_id() as usize };
        if let Some(item) = self.items.get(id) {
            item
        } else {
            self.get_slow(id, token)
        }
    }

    #[cold]
    fn get_slow<'a>(&'a self, id: usize, _token: &'a NoInterrupts) -> &'a T {
        todo!()
    }
}

fn hwtid() -> usize {
    unsafe { hw_thread_id() as usize }
}
