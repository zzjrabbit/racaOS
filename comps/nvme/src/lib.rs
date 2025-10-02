#![no_std]

extern crate alloc;

use component::{ComponentInitError, init_component};

mod allocator;

#[init_component(kthread)]
pub fn init() -> Result<(), ComponentInitError> {
    Ok(())
}
