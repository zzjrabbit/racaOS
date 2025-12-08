pub use int::enable_int;

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
