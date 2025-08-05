//! This crate provides customizable basic functionalities for operating systems written in Rust.

#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]

extern crate alloc;

pub use error::*;
pub use zodiac_macro::{global_allocator, main, panic_handler};

use crate::hal::disable_interrupts;

mod boot;
#[doc(hidden)]
pub mod console;
mod error;
/// Functions to access framebuffer. 
/// Warning: these interfaces are not thread-safe.
pub mod framebuffer;
/// Safe wrappers to access hardware.
#[cfg(target_arch = "x86_64")]
#[path = "hal/x86_64/mod.rs"]
pub mod hal;
/// Logging support.
pub mod logger;
/// Safe memory management.
pub mod mem;
mod panic;
/// Task structure definition and scheduling.
pub mod task;
/// Irq allocation and page fault handling.
pub mod trap;

fn init() {
    disable_interrupts();
    mem::init();
    logger::init();
    hal::init();
}
