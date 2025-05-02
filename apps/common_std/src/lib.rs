#![no_std]
#![feature(macro_metavar_expr)]
#![feature(alloc_error_handler)]

pub mod debug;
mod error;
pub mod fb;
mod heap;
pub mod hw;
pub mod ipc;
pub mod memory;
pub mod right;
mod syscall;
pub mod task;

use alloc::format;
pub use error::*;

extern crate alloc;

pub fn dummy() {}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    crate::debug::debug(&format!("panic: {}", info)).unwrap();
    loop {}
}

const ARG_HANDLE: u32 = 0x3;

pub const INVALID_HANDLE: u32 = u32::MAX;
