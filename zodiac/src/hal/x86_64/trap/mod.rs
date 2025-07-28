use crate::{
    hal::context::{CpuException, TrapFrame},
    trap::IRQ_MANAGER,
};

pub(super) mod gdt;
pub(super) mod idt;

#[unsafe(no_mangle)]
pub extern "C" fn rust_entry(frame: &mut TrapFrame) {
    if let Some(cpu_exception) = CpuException::new(frame.int_num, frame.error_code) {
        log::warn!("CPU Exception: {:x?}", cpu_exception);
        log::warn!("Trap frame: {:x?}", frame);

        loop {}
    } else {
        IRQ_MANAGER.handle_irq(frame);
    }
}
