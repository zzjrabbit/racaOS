#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use mostd::{
    entry,
    mem::{CachePolicy, MMUFlags, PageProperty, PageSize, Privilege},
};

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
fn main() {
    let vmar = Vmar::new();
    let (address, _) = vmar
        .map_with_alloc(
            1024,
            PageProperty::new(
                MMUFlags::READ | MMUFlags::WRITE,
                CachePolicy::CacheCoherent,
                Privilege::KernelOnly,
            ),
        )
        .unwrap();
    vmar.write_val(address, &0x114514u64);

    let val: u64 = vmar.read_val(address).unwrap();
    assert_eq!(val, 0x114514u64);
}
