use super::instr::{rdmsr, wrmsr};

#[inline]
pub unsafe fn init_hw_thread(thrd: usize) {
    wrmsr(0xc0000103, thrd as u64);
}

#[inline]
pub unsafe fn hw_thread_id() -> usize {
    rdmsr(0xc0000103) as usize
}
