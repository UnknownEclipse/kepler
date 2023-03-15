#![no_std]
#![feature(const_trait_impl, ptr_sub_ptr, const_option_ext, step_trait, const_try)]

pub use crate::{
    frame::Frame,
    frame_allocator::{FrameAllocError, FrameAllocator},
    page::{Page, PageSize, Size4KiB},
    page_table::{Caching, MapOptions, PageLookupError, PageTable, PageTableError},
    phys_addr::PhysAddr,
    virt_addr::VirtAddr,
    virt_region::VirtRegion,
};

mod frame;
mod frame_allocator;
mod page;
mod page_table;
mod phys_addr;
mod virt_addr;
mod virt_region;

fn align_down(addr: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two());
    addr & !(align - 1)
}

fn align_up(addr: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two());
    ((addr + align - 1) / align) * align
}
