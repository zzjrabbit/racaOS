#![no_std]

extern crate alloc;

mod mm;
mod time;

use component::{ComponentInitError, init_component};
pub use mm::*;
pub use time::*;

#[init_component]
pub fn init() -> Result<(), ComponentInitError> {
    Ok(())
}
