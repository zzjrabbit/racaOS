use x86_64::VirtAddr;

use crate::{
    hal::{
        context::{CpuException, TrapFrame},
        cpu::Cpu,
        smp::CPUS,
    },
    mem::VirtualAddress,
    trap::IRQ_MANAGER,
};

pub(super) mod gdt;
pub(super) mod idt;

#[unsafe(no_mangle)]
extern "C" fn rust_entry(frame: &mut TrapFrame) {
    if let Some(cpu_exception) = CpuException::new(frame.int_num, frame.error_code) {
        log::warn!(
            "CPU Exception on {}: {:x?}",
            Cpu::current().id(),
            cpu_exception
        );
        log::warn!("Trap frame: {:x?}", frame);

        loop {}
    } else {
        IRQ_MANAGER.handle_irq(frame);
    }
}

pub(crate) fn set_kernel_stack(stack: VirtualAddress) {
    CPUS.with_cpu_info_mut(Cpu::current(), |cpu_info| {
        cpu_info.set_ring0_rsp(VirtAddr::new(stack as u64))
    });
}
