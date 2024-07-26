#![no_std]
#![feature(naked_functions)]

pub extern crate alloc;

pub mod debug;
pub mod fs;
pub mod io;
pub mod mm;
pub mod task;

use core::panic::PanicInfo;
pub use core::*;
use x86_64::instructions::hlt;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("User Panic:{}", info);
    loop {}
}

#[naked]
extern "C" fn syscall(
    _id: u64,
    _arg1: usize,
    _arg2: usize,
    _arg3: usize,
    _arg4: usize,
    _arg5: usize,
) -> usize {
    unsafe {
        core::arch::asm!(
            "mov rax, rdi",
            "mov rdi, rsi",
            "mov rsi, rdx",
            "mov rdx, rcx",
            "mov r10, r8",
            "mov r8, r9",
            "syscall",
            "ret",
            options(noreturn)
        )
    }
}

/// Stop and wait for something.
/// Now it uses assembly instruction "hlt".
/// We will change to "sleep" syscall in the future.
pub fn pause() {
    hlt();
}

extern "C" {
    fn main();
}

#[no_mangle]
pub unsafe extern "sysv64" fn _start() -> ! {
    //let _ = crate::io::open("/dev/console", crate::io::OpenType::Write);

    main();

    loop {}
}
