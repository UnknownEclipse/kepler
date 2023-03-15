use core::mem::MaybeUninit;

use spin::Lazy;
use x86_64::structures::tss::TaskStateSegment;

pub static TSS: Lazy<TaskStateSegment> = Lazy::new(|| unsafe { build_tss() });

unsafe fn build_tss() -> TaskStateSegment {
    static mut EXCEPTION_STACK: [MaybeUninit<u8>; 8192] = MaybeUninit::uninit_array();
    static mut PRIVILEGE_STACK: [MaybeUninit<u8>; 8192] = MaybeUninit::uninit_array();

    let top = EXCEPTION_STACK.as_mut_ptr_range().end;

    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[0] = x86_64::VirtAddr::from_ptr(top);
    tss.privilege_stack_table[0] =
        x86_64::VirtAddr::from_ptr(PRIVILEGE_STACK.as_mut_ptr_range().end);
    tss
}
