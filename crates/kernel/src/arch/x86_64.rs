pub use self::cpu::CpuId;

pub mod cpu;
pub mod gdt;
pub mod hpet;
pub mod idt;
pub mod syscall;
pub mod tss;
pub use idt::send_ipi;

pub fn init() {
    unsafe {
        gdt::init();
        idt::init();
    }
}

pub mod interrupts {
    use x86_64::instructions::{
        hlt,
        interrupts::{self, enable_and_hlt},
    };

    #[inline]
    pub unsafe fn enable() {
        interrupts::enable();
    }

    #[inline]
    pub unsafe fn disable() {
        interrupts::disable();
    }

    #[inline]
    pub unsafe fn wait() {
        hlt();
    }

    #[inline]
    pub unsafe fn enable_and_wait() {
        enable_and_hlt();
    }

    #[inline]
    pub unsafe fn are_enabled() -> bool {
        interrupts::are_enabled()
    }
}
