#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use mostd::{entry, mem::PageSize};

pub use vmar::*;
pub use vmo::*;

pub const PAGE_SIZE: usize = PageSize::Size4K as usize;

mod vmar;
mod vmo;

pub const fn align_down_by_page_size(addr: usize) -> usize {
    addr / PAGE_SIZE * PAGE_SIZE
}

pub const fn align_up_by_page_size(addr: usize) -> usize {
    align_down_by_page_size(addr + PAGE_SIZE - 1)
}

#[entry(memory)]
fn main() {}
