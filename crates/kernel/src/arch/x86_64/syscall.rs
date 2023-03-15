use core::arch::asm;

use hal::vm_types::VirtAddr;

// pub unsafe fn enable_syscall() {}
#[naked]
pub unsafe extern "C" fn sysret(
    rip: /* edi */ VirtAddr,
    rsp: /* esi */ VirtAddr,
    rflags: /* edx */ usize,
) {
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
        
        mov ecx, edi
        mov r11, 0x202
        sysret",
        options(noreturn)
    );
}
