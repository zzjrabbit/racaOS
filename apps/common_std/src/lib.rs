#![no_std]
#![feature(macro_metavar_expr)]
#![feature(alloc_error_handler)]

pub mod debug;
mod error;
pub mod fb;
pub mod handle;
mod heap;
pub mod hw;
pub mod ipc;
pub mod memory;
pub mod right;
pub mod signal;
mod syscall;
pub mod task;

pub use error::*;

extern crate alloc;

pub fn dummy() {}

pub const ARG_HANDLE: u32 = 0x3;

pub const INVALID_HANDLE: u32 = u32::MAX;

pub fn exit(code: i64) -> ! {
    syscall!(@noret 26, code);
}
