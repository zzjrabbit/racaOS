#![no_std]

extern crate alloc;

pub use boot::{kernel_base, kernel_file};
pub use error::*;
pub use zodiac_macro::*;

use crate::arch::enable_int;

mod acpi;
pub mod arch;
mod boot;
mod error;
pub mod framebuffer;
pub mod io;
pub mod irq;
pub mod logger;
pub mod mem;
mod panic;
pub mod sync;
pub mod task;
pub mod timer;

fn init() {
    mem::init();
    arch::init();
    logger::init();

    #[cfg(feature = "smp")]
    arch::init_smp();

    enable_int();
}
