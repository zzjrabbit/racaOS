use x86_64::{
    VirtAddr,
    registers::{
        control::{Cr0, Cr0Flags, Cr4, Cr4Flags},
        model_specific::{FsBase, GsBase, KernelGsBase},
    },
};

use crate::{
    hal::{cpu::Cpu, smp::CPUS},
    mem::VirtualAddress,
    trap::Irq,
};

pub mod bus;
/// Context structures.
pub mod context;
/// CPU structures.
pub mod cpu;
/// Device structures.
pub mod device;
/// IRQs, power management and others.
pub mod kernel;
/// Safe memory management wrappers.
pub mod mem;
mod smp;
/// Timer.
pub mod timer;
/// Safe wrappers for interrupts and syscalls.
pub mod trap;

/// Do something without interrupts.
pub fn without_interrupts<R>(function: impl FnMut() -> R) -> R {
    x86_64::instructions::interrupts::without_interrupts(function)
}

/// Enable interrupts.
pub fn enable_interrupts() {
    x86_64::instructions::interrupts::enable();
}

/// Disable interrupts.
pub fn disable_interrupts() {
    x86_64::instructions::interrupts::disable();
}

/// Get the number of CPUs.
pub fn cpu_num() -> usize {
    smp::CPUS.len()
}

impl Cpu {
    pub(crate) fn trigger_save_context(&self) {
        if Cpu::current() == *self {
            unsafe {
                core::arch::asm!("int 0x20");
            }
        } else {
            self.send_ipi(Irq::from_vector(0x20));
        }
    }
}

pub fn write_fs(fs: VirtualAddress) {
    FsBase::write(VirtAddr::new(fs as u64));
}

pub fn write_gs(gs: VirtualAddress) {
    GsBase::write(VirtAddr::new(gs as u64));
}

pub(crate) fn init() {
    smp::CPUS.load(Cpu::bsp());
    trap::idt::init();
    init_sse();
    kernel::init();
    timer::init();
    trap::init();
    bus::init();

    smp::CPUS.with_cpu_info_mut(Cpu::bsp(), |info| {
        KernelGsBase::write(VirtAddr::from_ptr(info.tss_mut()));
        log::info!("kernel gs base: {:x}", KernelGsBase::read());
    });
    GsBase::write(VirtAddr::zero());

    smp::CPUS.init_ap();
}

fn init_sse() {
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

    smp::CPUS.with_cpu_info_mut(Cpu::current(), |info| {
        KernelGsBase::write(VirtAddr::from_ptr(info.tss_mut()));
    });
    GsBase::write(VirtAddr::zero());

    log::debug!("Application Processor {} started", smp_info.id);

    crate::task::ap_init();

    loop {
        x86_64::instructions::hlt();
    }
}
