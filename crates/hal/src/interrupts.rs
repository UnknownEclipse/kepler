pub use crate::arch::interrupts::{
    ExceptionEntry, ExceptionHandler, InterruptEntry, InterruptHandler, InterruptTable,
    PageFaultError, StackFrame,
};
use crate::intrin::{disable_interrupts, enable_interrupts, interrupts_are_enabled};

#[derive(Debug)]
pub struct NoInterrupts(());

#[inline]
pub fn without<F, R>(f: F) -> R
where
    F: FnOnce(&mut NoInterrupts) -> R,
{
    unsafe {
        let were_enabled = are_enabled();
        if were_enabled {
            disable();
        }
        let mut token = NoInterrupts(());
        let result = f(&mut token);
        if were_enabled {
            enable();
        }
        result
    }
}

pub fn are_enabled() -> bool {
    unsafe { interrupts_are_enabled() }
}

/// # Safety
/// Force-enabling interrupts can be dangerous if an operation is depending on them
/// being disabled. Typically anything that is bound to a [NoInterrupts] token will cause
/// spectacular issues. Be careful with this!
pub unsafe fn enable() {
    enable_interrupts();
}

/// # Safety
/// There's not anything immediately dangerous about forcibly disabling interrupts, but
/// given that it's an intrinsic... just be careful.
pub unsafe fn disable() {
    disable_interrupts();
}
