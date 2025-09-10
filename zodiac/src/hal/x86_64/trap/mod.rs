use x86_64::VirtAddr;

use crate::{
    hal::{
        context::{CpuException, TrapFrame},
        cpu::Cpu,
        smp::CPUS,
    },
    mem::VirtualAddress,
    task::schedule,
    trap::IRQ_MANAGER,
};

pub(super) mod gdt;
pub(super) mod idt;
pub(super) mod syscall;

pub(crate) use syscall::init;

#[unsafe(no_mangle)]
extern "C" fn rust_entry(frame: &mut TrapFrame) {
    if let Some(cpu_exception) = CpuException::new(frame.int_num, frame.error_code) {
        log::warn!(
            "CPU Exception on {}: {:x?}",
            Cpu::current().id(),
            cpu_exception
        );
        log::warn!("Trap frame: {:#x?}", frame);

        loop {
            x86_64::instructions::hlt();
        }
    } else {
        if frame.int_num == 32 {
            schedule(frame);
            return;
        }
        IRQ_MANAGER.handle_irq(frame);
    }
}

pub(crate) fn get_kernel_stack() -> VirtualAddress {
    CPUS.with_cpu_info(Cpu::current(), |cpu_info| {
        cpu_info.get_ring0_rsp().as_u64() as VirtualAddress
    })
}

pub(crate) fn set_kernel_stack(stack: VirtualAddress) {
    CPUS.with_cpu_info_mut(Cpu::current(), |cpu_info| {
        cpu_info.set_ring0_rsp(VirtAddr::new(stack as u64))
    });
}

pub(crate) fn get_user_stack() -> VirtualAddress {
    CPUS.with_cpu_info(Cpu::current(), |cpu_info| {
        cpu_info.get_ring3_rsp().as_u64() as VirtualAddress
    })
}

pub(crate) fn set_user_stack(stack: VirtualAddress) {
    CPUS.with_cpu_info_mut(Cpu::current(), |cpu_info| {
        cpu_info.set_ring3_rsp(VirtAddr::new(stack as u64))
    });
}
