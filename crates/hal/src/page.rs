pub use hal_core::page::{PageSize, Size4KiB};

use crate::arch::addr::{Phys, Virt};

pub type Page<S> = hal_core::page::Page<Virt, S>;
pub type Frame<S> = hal_core::page::Page<Phys, S>;
