use core::arch::asm;
use spin::Mutex;
use x86_64::registers::model_specific::{Efer, EferFlags};
use x86_64::registers::model_specific::{LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;
use x86_64::VirtAddr;

use crate::arch::gdt::Selectors;

pub fn init() {
    let handler_addr = syscall_handler as *const () as u64;

    SFMask::write(RFlags::INTERRUPT_FLAG);
    LStar::write(VirtAddr::new(handler_addr as u64));

    let (code_selector, data_selector) = Selectors::get_kernel_segments();
    let (user_code_selector, user_data_selector) = Selectors::get_user_segments();

    Star::write(
        user_code_selector,
        user_data_selector,
        code_selector,
        data_selector,
    )
    .unwrap();

    unsafe {
        Efer::write(Efer::read() | EferFlags::SYSTEM_CALL_EXTENSIONS);
    }
}

#[naked]
extern "C" fn syscall_handler() {
    unsafe {
        asm!(
            "push rcx",
            "push r11",
            "push rbp",
            "push rbx",
            "push r12",
            "push r13",
            "push r14",
            "push r15",

            // Move the 4th argument in r10 to rcx to fit the C ABI
            "mov rcx, r10",
            "call {syscall_handle_fn}",

            "pop r15",
            "pop r14",
            "pop r13",
            "pop r12",
            "pop rbx",
            "pop rbp",
            "pop r11",
            "pop rcx",
            "sysretq",
            syscall_handle_fn = sym syscall_handle_fn,
            options(noreturn)
        );
    }
}

#[allow(unused_variables)]
pub extern "C" fn syscall_handle_fn(
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) -> usize {
    let syscall_number_raw: usize;
    unsafe { asm!("mov {0}, rax", out(reg) syscall_number_raw) };

    SYSCALL_HANDLER.lock()(syscall_number_raw, arg1, arg2, arg3, arg4, arg5, arg6)
}

fn tmp_syscall_handler(
    _idx: usize,
    _arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
    _arg6: usize,
) -> usize {
    0
}

pub type SyscallHandlerFn = fn(
    idx: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) -> usize;

static SYSCALL_HANDLER: Mutex<SyscallHandlerFn> = Mutex::new(tmp_syscall_handler);

pub fn regist_syscall_handler(handler: SyscallHandlerFn) {
    *SYSCALL_HANDLER.lock() = handler;
}
