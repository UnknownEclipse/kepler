#![no_std]

pub mod addr;
pub mod interrupts;
pub mod intrin;
pub mod page;
pub mod random;
pub mod region;
pub mod task;

#[cfg(target_arch = "x86")]
pub mod x86;
#[cfg(target_arch = "x86_64")]
pub mod x86_64;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86_common;

// pub use hal_x86_64 as x86_64;
use hal_x86_64 as arch;
