pub use crate::arch::intrin::{
    breakpoint, disable_interrupts, enable_interrupts, enable_interrupts_and_halt, halt,
    interrupts_are_enabled,
};

pub unsafe fn core_count() -> usize {
    todo!()
}

pub unsafe fn core_id() -> usize {
    todo!()
}
