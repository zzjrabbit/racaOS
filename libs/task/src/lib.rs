#![no_std]

extern crate alloc;

mod clone;
mod process;
mod scheduler;
mod syscall;
mod thread;
mod trap;
mod signal;

pub use clone::*;
use component::{ComponentInitError, init_component};
pub use process::*;
pub use thread::*;
pub use signal::*;

#[init_component]
pub fn init() -> Result<(), ComponentInitError> {
    trap::init();
    scheduler::init();
    Ok(())
}
