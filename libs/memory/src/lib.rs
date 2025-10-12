#![no_std]

extern crate alloc;

mod vmar;
mod vmo;

use component::{ComponentInitError, init_component};
use ostd::mm::PAGE_SIZE;
pub use vmar::*;
pub use vmo::*;

pub const fn align_down_by_page_size(addr: usize) -> usize {
    addr / PAGE_SIZE * PAGE_SIZE
}

pub const fn align_up_by_page_size(addr: usize) -> usize {
    align_down_by_page_size(addr + PAGE_SIZE - 1)
}

#[init_component]
pub fn init() -> Result<(), ComponentInitError> {
    Ok(())
}
