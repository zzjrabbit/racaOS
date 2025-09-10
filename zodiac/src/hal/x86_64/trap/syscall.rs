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

pub fn init() {
    const RFLAGS_MASK: u64 = 0x47700;
    SFMask::write(RFlags::from_bits(RFLAGS_MASK).unwrap());
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
        "swapgs",
        "mov gs:12, rsp",
        "mov rsp, gs:4",
        "pop rsp",
        "add rsp, 0xb0",
        "push 0x000000000000001b",
        "push gs:12",
        "push r11",
        "push 0x0000000000000023",
        "push rcx",
        "sub rsp, 8",
        "push 0xffffffffffffffff",
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
        "mov rsp, gs:4",
        "add rsp, 8",
        "pop rbx",
        "pop rbp",
        "pop r12",
        "pop r13",
        "pop r14",
        "pop r15",
        "swapgs",
        "ret",
    );
}

core::arch::global_asm!(include_str!("syscall.asm"),);

unsafe extern "C" {
    pub(in crate::hal) fn syscall_return(context: &mut TrapFrame);
}
