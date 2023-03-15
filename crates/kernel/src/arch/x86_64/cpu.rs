use hal::{interrupts, task::init_hw_thread};
use x86_64::registers::model_specific::Msr;

use super::idt::LOCAL_APIC;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CpuId(pub(super) u32);

impl CpuId {
    pub fn get() -> Self {
        get()
    }
}

pub fn get() -> CpuId {
    unsafe { CpuId(Msr::new(0xc0000103).read() as u32) }
}

pub unsafe fn set_current_cpu_id(id: u32) {
    unsafe { Msr::new(0xc0000103).write(id.into()) }
}

pub fn init(core: usize) {
    unsafe {
        init_hw_thread(core);
        interrupts::without(|_| {
            let mut apic = LOCAL_APIC.get().unwrap().lock();
            apic.enable();
        });
    }
}
