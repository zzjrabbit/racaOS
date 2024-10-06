#![no_std]
#![allow(improper_ctypes)]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]
#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]

extern crate alloc;

pub mod arch;
pub mod device;
pub mod error;
pub mod memory;
pub mod module;
pub mod syscall;
pub mod task;

pub fn init() {
    memory::init_heap();
    device::log::init();
    arch::smp::CPUS.write().init_bsp();
    arch::interrupts::IDT.load();
    arch::smp::CPUS.write().init_ap();
    arch::apic::init();
    syscall::init();
    task::init();
    log::info!("racaOS intialization completed!");
}
