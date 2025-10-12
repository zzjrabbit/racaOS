#![no_std]

extern crate alloc;

use component::{ComponentInitError, init_component};

mod drivers;

#[init_component(kthread)]
pub fn init() -> Result<(), ComponentInitError> {
    drivers::init();
    log::info!("Network initialized.");
    Ok(())
}
