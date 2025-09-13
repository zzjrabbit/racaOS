#![no_std]
#![no_main]
#![deny(unsafe_code)]
#![feature(allocator_api)]

use core::panic::PanicInfo;

use zodiac::{
    hal::{disable_interrupts, enable_interrupts, halt},
    smp::set_ap_entry,
    task::TaskBuilder,
};

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

fn idle() -> ! {
    loop {
        halt();
    }
}

fn ap_entry() -> ! {
    disable_interrupts();
    let idle_task = TaskBuilder::default().entry(idle).build().unwrap();
    idle_task.spawn();
    enable_interrupts();
    loop {}
}

#[zodiac::main]
pub fn main() {
    disable_interrupts();

    trap::init();
    task::init();
    syscall::init();
    terminal::init();
    filesystem::init();
    drivers::init();

    set_ap_entry(ap_entry);

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

    let idle_task = TaskBuilder::default().entry(idle).build().unwrap();
    idle_task.spawn();

    let _hello = Process::new(
        include_bytes!("../../apps/hello.bin"),
        tty.clone(),
        tty.clone(),
        tty.clone(),
    )
    .unwrap();

    enable_interrupts();
    loop {}
}

#[zodiac::panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    log::error!("panic: {}", info);
    loop {
        core::hint::spin_loop();
    }
}
