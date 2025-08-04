use spin::Once;
use x86_64::{
    registers::{
        control::{Efer, EferFlags}, model_specific::{LStar, SFMask, Star}, rflags::RFlags
    }, structures::gdt::SegmentSelector, PrivilegeLevel, VirtAddr
};

use crate::hal::{cpu::Cpu, smp::CPUS};

pub type SyscallHandler = fn(
    syscall_id: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) -> isize;

static SYSCALL_HANDLER: Once<SyscallHandler> = Once::new();

pub fn set_syscall_handler(handler: SyscallHandler) {
    SYSCALL_HANDLER.call_once(|| handler);
}

pub fn init() {
    SFMask::write(RFlags::INTERRUPT_FLAG);
    LStar::write(VirtAddr::from_ptr(syscall_handler as *const ()));

    CPUS.with_cpu_info(Cpu::current(), |cpu_info| {
        Star::write(
            SegmentSelector::new(cpu_info.user_code_selector() as u16 >> 3, PrivilegeLevel::Ring3),
            SegmentSelector::new(cpu_info.user_data_selector() as u16 >> 3, PrivilegeLevel::Ring3),
            SegmentSelector::new(cpu_info.kernel_code_selector() as u16 >> 3, PrivilegeLevel::Ring0),
            SegmentSelector::new(cpu_info.kernel_data_selector() as u16 >> 3, PrivilegeLevel::Ring0),
        ).unwrap();
    });

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

#[allow(unused_variables)]
pub extern "C" fn syscall_matcher(
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) -> isize {
    let syscall_index: usize;
    unsafe { core::arch::asm!("mov {0}, rax", out(reg) syscall_index) };

    match SYSCALL_HANDLER.get() {
        Some(handler) => handler(syscall_index, arg1, arg2, arg3, arg4, arg5, arg6),
        None => panic!("No syscall handler"),
    }
}

