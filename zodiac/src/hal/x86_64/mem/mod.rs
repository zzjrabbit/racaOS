use x86_64::align_down;

mod paging;

pub use paging::*;

pub const PAGE_SIZE: usize = 4096;

pub const KERNEL_ASPACE_BASE: usize = 0xffff_ff80_0000_0000;
pub const KERNEL_ASPACE_SIZE: usize = 0x0000_0010_0000_0000;
pub const USER_ASPACE_BASE: usize = 0x0000_0000_0400_0000;
pub const USER_ASPACE_SIZE: usize = KERNEL_ASPACE_BASE - USER_ASPACE_BASE;

pub fn align_down_by_page_size(address: usize) -> usize {
    align_down(address as u64, PAGE_SIZE as u64) as usize
}
