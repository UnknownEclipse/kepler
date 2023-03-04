use core::arch::global_asm;

extern "C" {
    pub fn context_switch(old: *mut *mut Context, new: *mut Context);
}

global_asm!(
    "context_switch:
        push r15
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
        ret"
);

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
    pub fn with_target(f: extern "C" fn() -> !) -> Self {
        Self {
            rbx: 0,
            rbp: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip: f as usize,
        }
    }
}
