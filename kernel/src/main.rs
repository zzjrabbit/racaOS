#![no_std]
#![no_main]
#![forbid(unsafe_code)]

use core::panic::PanicInfo;

use alloc::format;
use zodiac::{
    mem::{MMUFlags, PageSize, PhysicalMemoryAllocOptions, VirtualMemorySpace},
    task::{Process, ProcessBuilder, ThreadBuilder, start_schedule},
};

extern crate alloc;

mod heap;
mod scheduler;
mod terminal;
mod syscall;

fn thread_a() -> ! {
    let mut id = 0;
    loop {
        zodiac::print!("test {}\n", id);
        terminal::terminal_write(format!("test {}\n", id));
        id += 1;
        for _ in 0..10000000 {
            core::hint::spin_loop();
        }
    }
}

#[zodiac::main]
pub fn main() {
    scheduler::init();
    syscall::init();
    terminal::init();
    log::info!("Zodiac Initialize done, entering kernel.");

    log::info!(
        "total time: {}s",
        zodiac::hal::timer::elapsed().as_secs_f64()
    );

    let thread = ThreadBuilder::default()
        .entry(thread_a)
        .kernel_mode()
        .process(Process::kernel())
        .build()
        .unwrap();
    thread.spawn();

    const USER_STACK_END: usize = 0x7fffffff0000;
    const USER_STACK_SIZE: usize = 256 * 1024;

    let binary = include_bytes!("../../test");
    let vm_space = VirtualMemorySpace::new_user();

    let stack_address = USER_STACK_END - USER_STACK_SIZE;

    let physical_memory = PhysicalMemoryAllocOptions::default()
        .count(USER_STACK_SIZE / PageSize::Size4K as usize)
        .allocate()
        .unwrap();
    vm_space
        .cursor(stack_address, PageSize::Size4K)
        .unwrap()
        .map(
            &physical_memory,
            MMUFlags::READ | MMUFlags::WRITE | MMUFlags::USER,
        )
        .unwrap();
    let entry = vm_space.binary_file_mapper().map(binary).unwrap();

    let process = ProcessBuilder::default()
        .vm_space(vm_space)
        .build()
        .unwrap();
    let thread = ThreadBuilder::default()
        .entry(entry)
        .process(process.clone())
        .stack(USER_STACK_END)
        .build()
        .unwrap();
    thread.spawn();

    start_schedule();

    loop {}
}

#[zodiac::panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    log::error!("panic: {}", info);
    loop {}
}
