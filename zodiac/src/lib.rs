#![no_std]

extern crate alloc;

pub use boot::{kernel_base, kernel_file};
pub use error::*;
pub use zodiac_macro::*;

mod acpi;
pub mod arch;
mod boot;
mod error;
pub mod framebuffer;
pub mod logger;
pub mod mem;
mod panic;

fn init() {
    mem::init();
    arch::init();
    logger::init();
}
