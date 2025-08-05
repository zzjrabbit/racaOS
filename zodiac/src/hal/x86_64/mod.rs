use x86_64::registers::control::{Cr0, Cr0Flags, Cr4, Cr4Flags};

use crate::{hal::{cpu::Cpu, kernel::apic_timer_irq, smp::CPUS}, trap::Irq};

pub mod context;
pub mod cpu;
pub mod device;
pub mod kernel;
pub mod mem;
mod smp;
pub mod timer;
pub mod trap;

pub fn without_interrupts<R>(function: impl Fn() -> R) -> R {
    x86_64::instructions::interrupts::without_interrupts(function)
}

pub fn enable_interrupts() {
    x86_64::instructions::interrupts::enable();
}

pub fn disable_interrupts() {
    x86_64::instructions::interrupts::disable();
}

pub fn cpu_num() -> usize {
    smp::CPUS.len()
}

impl Cpu {
    pub fn trigger_schedule(&self) {
        if Cpu::current() == *self {
            unsafe {
                core::arch::asm!("int 0x20");
            }
        } else {
            self.send_ipi(Irq::from_vector(0x20));
        }
    }
}

pub fn init() {
    smp::CPUS.load(Cpu::bsp());
    trap::idt::init();
    init_sse();
    kernel::init();
    timer::init();
    trap::init();

    smp::CPUS.init_ap();
}

pub fn init_sse() {
    let mut cr0 = Cr0::read();
    cr0.remove(Cr0Flags::EMULATE_COPROCESSOR);
    cr0.insert(Cr0Flags::MONITOR_COPROCESSOR);
    unsafe { Cr0::write(cr0) };

    let mut cr4 = Cr4::read();
    cr4.insert(Cr4Flags::OSFXSR);
    cr4.insert(Cr4Flags::OSXMMEXCPT_ENABLE);
    unsafe { Cr4::write(cr4) };
}

unsafe extern "C" fn ap_entry(smp_info: &limine::mp::Cpu) -> ! {
    disable_interrupts();
    CPUS.load(Cpu::new(smp_info.lapic_id));
    trap::idt::init();

    init_sse();

    kernel::ap_init();
    timer::ap_init();
    trap::init();

    log::debug!("Application Processor {} started", smp_info.id);

    crate::task::ap_init();

    loop {
        x86_64::instructions::hlt();
    }
}
