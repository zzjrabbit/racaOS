use core::sync::atomic::Ordering;
use driver::apic::{APIC_INIT, CALIBRATED_TIMER_INITIAL, LAPIC};
use int::IDT;
use limine::mp::Cpu;
use smp::BSP_LAPIC_ID;
use smp::CPUS;
use x86_64::registers::control::{Cr0, Cr4};
use x86_64::registers::control::{Cr0Flags, Cr4Flags};

pub mod driver;
pub mod fb;
pub mod gdt;
pub mod int;
mod io_port;
mod mem;
pub mod smp;
mod syscall;

pub use io_port::*;
pub use mem::*;

use crate::task::scheduler::SCHEDULER_INIT;

pub fn init() {
    smp::CPUS.write().load(*BSP_LAPIC_ID);
    int::init();
    init_sse();
    smp::CPUS.write().init_ap();
    driver::acpi::init();
    driver::apic::init();
    syscall::init();
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

unsafe extern "C" fn ap_entry(smp_info: &Cpu) -> ! {
    CPUS.write().load(smp_info.lapic_id);
    IDT.load();

    init_sse();

    while !APIC_INIT.load(Ordering::SeqCst) {
        core::hint::spin_loop()
    }
    unsafe {
        LAPIC.lock().enable();
    }

    let timer_initial = CALIBRATED_TIMER_INITIAL.load(Ordering::SeqCst);
    unsafe {
        LAPIC.lock().set_timer_initial(timer_initial);
        LAPIC.lock().enable_timer();
    }

    syscall::init();

    while !SCHEDULER_INIT.load(Ordering::SeqCst) {
        core::hint::spin_loop()
    }
    log::debug!("Application Processor {} started", smp_info.id);
    x86_64::instructions::interrupts::enable();

    loop {
        x86_64::instructions::hlt();
    }
}
