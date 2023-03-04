#![no_std]
#![feature(const_trait_impl, ptr_sub_ptr, const_option_ext, step_trait, const_try)]

pub mod addr;
pub mod addr2;
pub mod alloc;
pub mod page;
pub mod region;

fn align_down(addr: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two());
    addr & !(align - 1)
}

fn align_up(addr: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two());
    addr & !(align - 1)
}
