#![no_std]

extern crate alloc;

pub use zodiac_macro::*;

mod acpi;
pub mod arch;
mod boot;
pub mod framebuffer;
pub mod logger;
pub mod mem;
mod panic;

fn init() {
    mem::init();
    arch::init();
    logger::init();
}
