#![no_std]

use component::{ComponentInitError, init_component};

#[init_component(kthread)]
pub fn init() -> Result<(), ComponentInitError> {
    Ok(())
}
