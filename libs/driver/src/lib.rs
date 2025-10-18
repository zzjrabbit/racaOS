#![no_std]

extern crate alloc;

mod mm;

use component::{ComponentInitError, init_component};
pub use mm::*;

#[init_component]
pub fn init() -> Result<(), ComponentInitError> {
    Ok(())
}
