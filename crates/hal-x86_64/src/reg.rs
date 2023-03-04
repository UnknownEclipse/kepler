//! Raw access to x86 registers

macro_rules! define_reg_ops {
    ($name:ident) => {
        pub mod $name {
            use core::arch::asm;

            #[inline]
            pub fn read() -> u64 {
                let value: u64;
                unsafe {
                    asm!(concat!("mov {}, ", stringify!($name)), out(reg) value, options(nomem, nostack, preserves_flags));
                }
                value
            }

            #[inline]
            pub unsafe fn write(value: u64) {
                asm!(concat!("mov ", stringify!($name), ", {}"), in(reg) value, options(nostack, preserves_flags));
            }
        }
    };
}

define_reg_ops!(rax);
define_reg_ops!(rbx);
define_reg_ops!(rcx);
define_reg_ops!(rdx);
define_reg_ops!(rsi);
define_reg_ops!(rdi);
define_reg_ops!(rsp);
define_reg_ops!(rbp);

define_reg_ops!(rip);

define_reg_ops!(cr0);
define_reg_ops!(cr2);
define_reg_ops!(cr3);
define_reg_ops!(cr4);
define_reg_ops!(cr8);

define_reg_ops!(dr0);
define_reg_ops!(dr1);
define_reg_ops!(dr2);
define_reg_ops!(dr3);
define_reg_ops!(dr6);
define_reg_ops!(dr7);

pub mod rflags {
    use x86_64::registers::rflags;

    #[inline]
    pub fn read() -> u64 {
        rflags::read_raw()
    }

    #[inline]
    pub unsafe fn write(value: u64) {
        rflags::write_raw(value);
    }
}
