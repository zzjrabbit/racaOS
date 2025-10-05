#![no_std]
#![no_main]
#![deny(unsafe_code)]
#![feature(allocator_api)]
#![feature(let_chains)]

use core::panic::PanicInfo;

use component::InitStage;
use ostd::{
    arch::qemu::{exit_qemu, QemuExitCode},
    boot::smp::register_ap_entry,
    cpu::CpuId,
    prelude::*,
    task::{halt_cpu, scheduler::enable_preemption_on_cpu},
};

use crate::{
    comps::components,
    filesystem::{open_file, FileType, Path},
    task::{create_kernel_thread, Process},
};

extern crate alloc;

mod comps;
mod drivers;
mod filesystem;
mod mem;
mod syscall;
mod task;
mod trap;

fn ap_entry() {
    create_kernel_thread(idle);

    enable_preemption_on_cpu();
}

fn idle() {
    println!("Running on CPU #{}!", CpuId::current_racy().as_usize());
    loop {
        halt_cpu();
    }
}

#[ostd::main]
pub fn kernel_main() {
    component::init_all(InitStage::Bootstrap, components()).unwrap();
    trap::init();
    task::init();
    syscall::init();
    filesystem::init();
    drivers::init();

    register_ap_entry(ap_entry);

    enable_preemption_on_cpu();

    log::info!("Zodiac Initialize done, entering kernel.");

    log::info!(
        "total time: {}s",
        ostd::timer::Jiffies::elapsed().as_duration().as_secs_f64()
    );

    create_kernel_thread(first_kernel_thread);
}

fn first_kernel_thread() {
    log::info!("Running on CPU #{}!", CpuId::current_racy().as_usize());
    component::init_all(InitStage::Kthread, components()).unwrap();

    let input = open_file(&Path::new("/"))
        .unwrap()
        .create("input.txt".into(), FileType::File)
        .unwrap();
    input.write_at(0, b"   Hello World, File!\n");

    let tty = open_file(&Path::new("/dev/tty")).unwrap();

    let hello = open_file(&Path::from("/part0/hello.bin")).unwrap();
    let mut buffer = alloc::vec![0u8; hello.len() as usize];
    hello.read_at(0, &mut buffer);

    let hello = Process::new(&buffer, tty.clone(), tty.clone(), tty.clone()).unwrap();

    while hello.exit_code().is_none() {
        halt_cpu();
    }

    log::error!("Init process exited.");

    loop {
        halt_cpu();
    }
}

#[ostd::panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    println!("panic: {}", info);
    exit_qemu(QemuExitCode::Failed);
    loop {
        core::hint::spin_loop();
    }
}
