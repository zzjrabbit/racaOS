pub(crate) use int::{disable_int, enable_int};

use crate::irq::DisabledLocalIrqGuard;

#[cfg(feature = "smp")]
mod boot;
pub(crate) mod context;
mod error;
mod int;
pub mod mem;
pub mod serial;
mod timer;

pub(crate) fn init() {
    serial::init();
    int::init();
}

#[cfg(feature = "smp")]
pub(crate) fn init_smp() {
    boot::init();
}

pub fn idle_ins() {
    if DisabledLocalIrqGuard::count() != 0 {
        panic!("Disable irq and idle would cause dead loop!");
    }
    unsafe {
        core::arch::asm!("idle 0");
    }
}

pub fn idle_loop() -> ! {
    loop {
        idle_ins();
    }
}

pub fn current_cpu() -> u64 {
    0
}

pub fn cpu_num() -> u64 {
    1
}
