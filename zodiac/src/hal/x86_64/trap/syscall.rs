use spin::Once;
use x86_64::{
    PrivilegeLevel, VirtAddr,
    registers::{
        control::{Efer, EferFlags},
        model_specific::{LStar, SFMask, Star},
        rflags::RFlags,
    },
    structures::gdt::SegmentSelector,
};

use crate::hal::{context::TrapFrame, cpu::Cpu, smp::CPUS};

pub type SyscallHandler = fn(frame: &mut TrapFrame) -> isize;

static SYSCALL_HANDLER: Once<SyscallHandler> = Once::new();

/// Set the syscall handler. So that it will be called when syscalls are invoked.
pub fn set_syscall_handler(handler: SyscallHandler) {
    SYSCALL_HANDLER.call_once(|| handler);
}

pub fn init() {
    SFMask::write(RFlags::INTERRUPT_FLAG);
    LStar::write(VirtAddr::from_ptr(syscall_handler as *const ()));

    CPUS.with_cpu_info(Cpu::current(), |cpu_info| {
        Star::write(
            SegmentSelector::new(
                cpu_info.user_code_selector() as u16 >> 3,
                PrivilegeLevel::Ring3,
            ),
            SegmentSelector::new(
                cpu_info.user_data_selector() as u16 >> 3,
                PrivilegeLevel::Ring3,
            ),
            SegmentSelector::new(
                cpu_info.kernel_code_selector() as u16 >> 3,
                PrivilegeLevel::Ring0,
            ),
            SegmentSelector::new(
                cpu_info.kernel_data_selector() as u16 >> 3,
                PrivilegeLevel::Ring0,
            ),
        )
        .unwrap();
    });

    unsafe {
        Efer::write(Efer::read() | EferFlags::SYSTEM_CALL_EXTENSIONS);
    }
}

#[unsafe(naked)]
unsafe extern "C" fn syscall_handler() {
    core::arch::naked_asm!(
        "push 24",
        "push rsp - 8",
        "pushfq",
        "push 8",
        "push {syscall_handler}",

        "add rsp, 16",

        "push rax",
        "push rcx",
        "push rdx",
        "push rdi",
        "push rsi",
        "push r8",
        "push r9",
        "push r10",
        "push r11",

        "push rbx",
        "push rbp",
        "push r12",
        "push r13",
        "push r14",
        "push r15",

        "mov rdi, rsp",

        "call {syscall_matcher}",

        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbp",
        "pop rbx",

        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "pop rsi",
        "pop rdi",
        "pop rdx",
        "pop rcx",
        "pop rax",

        "add rsp, 56",

        "sysretq",
        syscall_matcher = sym syscall_matcher,
        syscall_handler = sym syscall_handler,
    );
}

#[allow(unused_variables)]
pub extern "C" fn syscall_matcher(frame: &mut TrapFrame) -> isize {
    match SYSCALL_HANDLER.get() {
        Some(handler) => handler(frame),
        None => panic!("No syscall handler"),
    }
}
