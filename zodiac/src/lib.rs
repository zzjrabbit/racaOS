#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]

extern crate alloc;

pub use error::*;
pub use zodiac_macro::{global_allocator, main, panic_handler};

mod boot;
pub mod console;
mod error;
#[cfg(target_arch = "x86_64")]
#[path = "hal/x86_64/mod.rs"]
pub mod hal;
pub mod logger;
pub mod mem;
mod panic;
pub mod trap;

fn init() {
    mem::init();
    logger::init();
    hal::init();
    hal::enable_interrupts();
}
