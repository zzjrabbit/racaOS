#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(variant_count)]
#![feature(allocator_api)]

pub mod arch;
pub mod console;
pub mod device;
pub mod memory;
pub mod syscall;
pub mod task;

extern crate alloc;

pub fn init() {
    console::printk::init();
    console::log::init();
    arch::gdt::init();
    arch::interrupts::IDT.load();
    memory::init();
    arch::smp::init();
    arch::acpi::init();
    device::hpet::init();
    arch::apic::init();
    device::mouse::init();
    syscall::init();
    task::scheduler::init();
}

pub fn addr_of<T>(reffer: &T) -> usize {
    reffer as *const T as usize
}

pub fn ref_to_mut<T>(reffer: &T) -> &mut T {
    unsafe { &mut *(addr_of(reffer) as *const T as *mut T) }
}
