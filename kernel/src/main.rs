#![no_std]
#![no_main]
#![deny(unsafe_code)]
#![feature(allocator_api)]

use core::panic::PanicInfo;

use zodiac::task::start_schedule;

use crate::{
    filesystem::{FileType, Path, open_file},
    task::Process,
};

extern crate alloc;

mod drivers;
mod filesystem;
mod heap;
mod syscall;
mod task;
mod terminal;
mod trap;

#[zodiac::main]
pub fn main() {
    trap::init();
    task::init();
    syscall::init();
    terminal::init();
    filesystem::init();
    drivers::init();
    log::info!("Zodiac Initialize done, entering kernel.");

    log::info!(
        "total time: {}s",
        zodiac::hal::timer::elapsed().as_secs_f64()
    );

    let tty = open_file(&Path::new("/dev/tty")).unwrap();

    let input = open_file(&Path::new("/"))
        .unwrap()
        .create("input.txt".into(), FileType::File)
        .unwrap();
    input.write_at(0, b"   Hello World, File!\n");

    let _hello = Process::new(
        include_bytes!("../../apps/hello.bin"),
        tty.clone(),
        tty.clone(),
        tty.clone(),
    )
    .unwrap();

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
