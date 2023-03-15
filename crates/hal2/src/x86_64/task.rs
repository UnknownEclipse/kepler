use core::arch::asm;

pub use crate::x86_common::task::*;

#[naked]
pub unsafe extern "C" fn context_switch(old: *mut *mut Context, new: *mut Context) {
    asm!(
        "push r15
        push r14
        push r13
        push r12
        push rbp
        push rbx
        mov [rdi], rsp
        mov rsp, rsi
        pop rbx
        pop rbp
        pop r12
        pop r13
        pop r14
        pop r15
        mov rdi, r15
        ret",
        options(noreturn)
    );
}

extern "C" {
    fn ring3_entry();
}

#[naked]
pub unsafe extern "C" fn jump_ring3() {
    asm!(
        "
        mov rcx, 0xc0000082
        wrmsr
        mov rcx, 0xc0000080
        rdmsr
        or eax, 1
        wrmsr
        mov rcx, 0xc0000081
        rdmsr
        mov edx, 0x00180008
        wrmsr

        mov ecx, ring3_entry
        mov r11, 0x202
        sysretq",
        options(noreturn)
    );
}

unsafe fn enable_syscall() {}

#[naked]
pub unsafe extern "C" fn context_switch_and_enable_interrupts(
    old: *mut *mut Context,
    new: *mut Context,
) {
    asm!(
        "push r15
        push r14
        push r13
        push r12
        push rbp
        push rbx
        mov [rdi], rsp
        mov rsp, rsi
        pop rbx
        pop rbp
        pop r12
        pop r13
        pop r14
        pop r15
        mov rdi, r15
        sti
        ret",
        options(noreturn)
    );
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Context {
    rbx: usize,
    rbp: usize,
    r12: usize,
    r13: usize,
    r14: usize,
    r15: usize,
    rip: usize,
}

impl Context {
    pub fn with_initial(f: extern "C" fn(*mut ()) -> !, data: *mut ()) -> Self {
        Self {
            rbx: 0,
            rbp: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: data as usize,
            rip: f as usize,
        }
    }
}
