use spin::Once;
use x86_64::VirtAddr;

use crate::{
    hal::{
        context::{CpuException, TrapFrame},
        cpu::Cpu,
        mem::KERNEL_ASPACE_BASE,
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
pub use syscall::set_syscall_handler;

pub type PageFaultHandler = fn(&mut TrapFrame, CpuException);

static PAGE_FAULT_HANDLER: Once<PageFaultHandler> = Once::new();

pub fn set_user_page_fault_handler(handler: PageFaultHandler) {
    PAGE_FAULT_HANDLER.call_once(|| handler);
}

#[unsafe(no_mangle)]
extern "C" fn rust_entry(frame: &mut TrapFrame) {
    if let Some(cpu_exception) = CpuException::new(frame.int_num, frame.error_code) {
        if let CpuException::PageFault(_, _) = cpu_exception
            && frame.rip < KERNEL_ASPACE_BASE
            && let Some(handler) = PAGE_FAULT_HANDLER.get()
        {
            handler(frame, cpu_exception);
            return;
        }

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

pub(crate) fn set_kernel_stack(stack: VirtualAddress) {
    CPUS.with_cpu_info_mut(Cpu::current(), |cpu_info| {
        cpu_info.set_ring0_rsp(VirtAddr::new(stack as u64))
    });
}
