use x86_64::VirtAddr;
use x86_64::registers::model_specific::{Efer, EferFlags};
use x86_64::registers::model_specific::{LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;

use super::gdt::Selectors;
use crate::syscall::syscall_matcher;

pub fn init() {
    SFMask::write(RFlags::INTERRUPT_FLAG);
    LStar::write(VirtAddr::from_ptr(syscall_handler as *const ()));

    let (kernel_code, kernel_data) = Selectors::get_kernel_segments();
    let (user_code, user_data) = Selectors::get_user_segments();
    Star::write(user_code, user_data, kernel_code, kernel_data).unwrap();

    unsafe {
        Efer::write(Efer::read() | EferFlags::SYSTEM_CALL_EXTENSIONS);
    }
}

#[unsafe(naked)]
unsafe extern "C" fn syscall_handler() {
    core::arch::naked_asm!(
        "push rcx",
        "push r11",

        "mov rcx, r10",

        "call {syscall_matcher}",

        "pop r11",
        "pop rcx",
        "sysretq",
        syscall_matcher = sym syscall_matcher,
    );
}
