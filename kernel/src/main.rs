#![no_std]
#![no_main]
#![forbid(unsafe_code)]

use core::panic::PanicInfo;

use zodiac::task::start_schedule;

use crate::{
    filesystem::{FileType, Path, open_file},
    task::Process,
};

extern crate alloc;

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
    log::info!("Zodiac Initialize done, entering kernel.");

    log::info!(
        "total time: {}s",
        zodiac::hal::timer::elapsed().as_secs_f64()
    );

    let tty = open_file(&Path::new("/dev/tty")).unwrap();
    tty.write_at(0, b"Hello World, TTY!\n");

    let input = open_file(&Path::new("/"))
        .unwrap()
        .create("input.txt".into(), FileType::File)
        .unwrap();
    input.write_at(0, b"Hello World, File!\n");

    let _hello = Process::new(
        include_bytes!("../../target/x86_64-unknown-linux-musl/release/hello"),
        tty.clone(),
        tty.clone(),
        tty.clone(),
    );

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
