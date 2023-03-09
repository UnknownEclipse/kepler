#![allow(clippy::missing_safety_doc)]

pub mod rflags {
    use core::arch::asm;

    #[inline]
    pub unsafe fn read() -> usize {
        let value: usize;
        unsafe {
            asm!("pushfq; pop {}", out(reg) value, options(nomem, preserves_flags));
        }
        value
    }

    #[inline]
    pub unsafe fn write(value: usize) {
        asm!("push {}; popfq", in(reg) value, options(nomem, preserves_flags));
    }
}
