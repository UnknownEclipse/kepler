#![no_std]
#![feature(const_trait_impl, ptr_sub_ptr, const_option_ext, step_trait, const_try)]

pub use crate::{
    frame::Frame,
    page::{Page, PageSize, Size4KiB},
    phys_addr::PhysAddr,
    virt_addr::VirtAddr,
};

mod frame;
mod frame_allocator;
mod page;
mod page_table;
mod phys_addr;
mod virt_addr;

fn align_down(addr: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two());
    addr & !(align - 1)
}

fn align_up(addr: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two());
    addr & !(align - 1)
}
