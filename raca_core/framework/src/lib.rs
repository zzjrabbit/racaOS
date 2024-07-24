#![no_std]
#![feature(allocator_api)]
#![feature(naked_functions)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

use core::sync::atomic::{AtomicBool, Ordering};

extern crate alloc;

pub mod arch;
pub mod console;
pub mod drivers;
pub mod memory;
pub mod task;
pub mod terminal;
pub mod user;

static START_SCHEDULE: AtomicBool = AtomicBool::new(false);

pub fn init_framework() {
    memory::init();
    console::init();
    arch::smp::CPUS.lock().init_bsp();
    arch::interrupts::IDT.load();
    arch::smp::CPUS.lock().init_ap();
    arch::acpi::init();
    drivers::hpet::init();

    let mut lapic = arch::apic::get_lapic();
    unsafe {
        lapic.enable();
        arch::apic::calibrate_timer(&mut lapic);
        lapic.enable_timer();
    }

    arch::apic::init();
    drivers::mouse::init();
    drivers::pci::init();
    user::init();
    task::scheduler::init();
}

#[inline]
pub fn start_schedule() {
    START_SCHEDULE.store(true, Ordering::Relaxed);
    x86_64::instructions::interrupts::enable();
}

pub fn addr_of<T>(reffer: &T) -> usize {
    reffer as *const T as usize
}

pub fn ref_to_mut<T>(reffer: &T) -> &mut T {
    unsafe { &mut *(addr_of(reffer) as *const T as *mut T) }
}

pub fn ref_to_static<T>(reffer: &T) -> &'static T {
    unsafe { &*(addr_of(reffer) as *const T) }
}

#[macro_export]
macro_rules! unsafe_trait_impl {
    ($struct: ident, $trait: ident) => {
        unsafe impl $trait for $struct {}
    };
    ($struct: ident, $trait: ident, $life: tt) => {
        unsafe impl<$life> $trait for $struct<$life> {}
    };
}
