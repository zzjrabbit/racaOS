#![no_std]
#![no_main]
#![forbid(unsafe_code)]

use core::panic::PanicInfo;

use zodiac::task::start_schedule;

use crate::{filesystem::{open_file, Path}, task::Process};

extern crate alloc;

mod filesystem;
mod heap;
mod scheduler;
mod syscall;
mod task;
mod terminal;

#[zodiac::main]
pub fn main() {
    scheduler::init();
    syscall::init();
    terminal::init();
    filesystem::init();
    log::info!("Zodiac Initialize done, entering kernel.");

    log::info!(
        "total time: {}s",
        zodiac::hal::timer::elapsed().as_secs_f64()
    );
    
    let tty = open_file(&Path::new("/dev/tty")).unwrap();
    tty.write_at(0, b"Hello World, TTY!\n");

    let _hello = Process::new(include_bytes!("../../apps/hello.bin"), tty.clone(), tty.clone());

    start_schedule();
    unreachable!()
}

#[zodiac::panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    log::error!("panic: {}", info);
    loop {
        core::hint::spin_loop();
    }
}
