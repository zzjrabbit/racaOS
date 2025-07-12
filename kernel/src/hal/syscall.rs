use x86_64::VirtAddr;
use x86_64::registers::model_specific::{Efer, EferFlags, GsBase, KernelGsBase};
use x86_64::registers::model_specific::{LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;

use super::driver::apic::LAPIC;
use super::gdt::Selectors;
use super::smp::CPUS;
use crate::syscall::syscall_matcher;

pub fn init() {
    SFMask::write(RFlags::INTERRUPT_FLAG);
    LStar::write(VirtAddr::from_ptr(syscall_handler as *const ()));

    GsBase::write(VirtAddr::new(0));

    let address =
        VirtAddr::from_ptr(&CPUS.read().get(unsafe { LAPIC.lock().id() }).syscall_info as *const _);
    KernelGsBase::write(address);

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
        //"swapgs",

        "push rcx",
        "push r11",

        //"mov rcx, rsp",
        //"mov r11, gs:0",
        //"mov rsp, r11",
        //"push rcx",

        "mov rcx, r10",

        "call {syscall_matcher}",

        //"pop rcx",
        //"mov rsp,rcx",

        "pop r11",
        "pop rcx",

        //"swapgs",
        "sysretq",
        syscall_matcher = sym syscall_matcher,
    );
}
