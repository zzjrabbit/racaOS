#![no_std]
#![no_main]
#![forbid(unsafe_code)]

use core::panic::PanicInfo;

use alloc::format;
use zodiac::task::{Process, ThreadBuilder, start_schedule};

extern crate alloc;

mod heap;
mod scheduler;
mod terminal;

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

    start_schedule();

    loop {}
}

#[zodiac::panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    log::error!("panic: {}", info);
    loop {}
}
