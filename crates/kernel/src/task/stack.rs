use core::{cell::UnsafeCell, ptr::NonNull};

use hal::task::context_switch;

use crate::error::KernResult;

pub struct Stack {
    ptr: NonNull<[u8]>,
    top: UnsafeCell<NonNull<u8>>,
}

unsafe impl Send for Stack {}
unsafe impl Sync for Stack {}

pub fn allocate_kernel_stack() -> KernResult<Stack> {
    todo!()
}

pub unsafe fn stack_switch(old: &Stack, new: &Stack) {
    let old = old.top.get().cast();
    let new = new.top.get().read().as_ptr().cast();
    context_switch(old, new);
}
