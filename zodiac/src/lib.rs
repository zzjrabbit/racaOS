#![no_std]
#![feature(abi_x86_interrupt)]

extern crate alloc;

pub use zodiac_macro::{main, panic_handler, global_allocator};
pub use error::*;

mod boot;
pub mod console;
mod error;
#[cfg(target_arch = "x86_64")]
#[path = "hal/x86_64/mod.rs"]
pub mod hal;
pub mod logger;
pub mod mem;
mod panic;

fn init() {
    mem::init();
    logger::init();
    hal::init();
}
