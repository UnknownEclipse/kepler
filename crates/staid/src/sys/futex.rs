use core::sync::atomic::AtomicU32;

use super::syscall::syscall3;

pub fn wait(atomic: &AtomicU32, value: u32) {
    unsafe {
        futex(atomic, FUTEX_OP_WAIT, value as usize);
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn wake(atomic: *const AtomicU32) -> bool {
    unsafe { futex(atomic, FUTEX_OP_WAKE_ONE, 0) != 0 }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn wake_all(atomic: *const AtomicU32) -> usize {
    unsafe { futex(atomic, FUTEX_OP_WAKE_ALL, 0) as usize }
}

unsafe extern "C" fn futex(atomic: *const AtomicU32, op: u32, arg0: usize) -> isize {
    syscall3(SYS_FUTEX, atomic as usize, op as usize, arg0)
}

const SYS_FUTEX: u32 = 1;

const FUTEX_OP_WAIT: u32 = 0;
const FUTEX_OP_WAKE_ALL: u32 = 1;
const FUTEX_OP_WAKE_ONE: u32 = 2;
