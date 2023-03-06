#![no_std]
#![feature(
    const_trait_impl,
    never_type,
    trait_alias,
    abi_x86_interrupt,
    sync_unsafe_cell,
    const_try,
    naked_functions
)]

pub mod addr;
pub mod features;
pub mod interrupts;
pub mod intrin;
pub mod paging;
pub mod port;
pub mod reg;
pub mod task;
pub mod vm;
