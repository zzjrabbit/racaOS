#![no_std]
#![feature(allocator_api)]
#![feature(naked_functions)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

pub mod arch;
pub mod console;
pub mod drivers;
pub mod memory;
pub mod task;
pub mod user;

pub fn init_framework() {
    console::init();
    memory::init();
    arch::init_acpi();
    drivers::init();
    arch::basic_init();
    arch::init_apic();
    task::init();
}

pub fn addr_of<T>(reffer: &T) -> usize {
    reffer as *const T as usize
}

pub fn ref_to_mut<T>(reffer: &T) -> &mut T {
    unsafe { &mut *(addr_of(reffer) as *const T as *mut T) }
}

pub fn ref_to_static<T>(reffer: &T) -> &'static T {
    unsafe { & *(addr_of(reffer) as *const T)}
}
