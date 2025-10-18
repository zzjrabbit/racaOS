#![no_std]
#![feature(result_flattening)]
#![feature(int_roundings)]

extern crate alloc;

mod clone;
mod process;
mod scheduler;
mod signal;
mod syscall;
mod thread;
mod trap;

pub use clone::*;
use component::{ComponentInitError, init_component};
pub use process::*;
pub use signal::*;
pub use thread::*;

#[init_component]
pub fn init() -> Result<(), ComponentInitError> {
    trap::init();
    scheduler::init();
    Ok(())
}
