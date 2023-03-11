#![allow(clippy::missing_safety_doc)]

pub mod flags {
    use core::arch::asm;

    #[inline]
    pub unsafe fn read() -> usize {
        let value: usize;
        unsafe {
            asm!("pushf; pop {}", out(reg) value, options(nomem, preserves_flags));
        }
        value
    }

    #[inline]
    pub unsafe fn write(value: usize) {
        asm!("push {}; popf", in(reg) value, options(nomem, preserves_flags));
    }
}

pub mod eflags {
    use core::arch::asm;

    #[inline]
    pub unsafe fn read() -> usize {
        let value: usize;
        unsafe {
            asm!("pushfd; pop {}", out(reg) value, options(nomem, preserves_flags));
        }
        value
    }

    #[inline]
    pub unsafe fn write(value: usize) {
        asm!("push {}; popfd", in(reg) value, options(nomem, preserves_flags));
    }
}

// TODO: Move to x86-64
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

pub mod cr3 {
    use core::arch::asm;

    #[inline]
    pub unsafe fn read() -> usize {
        let value: usize;
        asm!("mov {}, cr3", out(reg) value, options(nomem, nostack, preserves_flags));
        value
    }

    #[inline]
    pub unsafe fn write(value: usize) {
        asm!("mov cr3, {}", in(reg) value, options(nomem, nostack, preserves_flags));
    }
}

pub mod cs {
    use core::arch::asm;

    #[inline]
    pub unsafe fn read() -> u16 {
        let value: u16;
        asm!("mov {0:x}, cs", out(reg) value, options(nomem, nostack, preserves_flags));
        value
    }
}
