mod paging;

pub(crate) use paging::*;

/// The base of kernel address space.
pub const KERNEL_ASPACE_BASE: usize = 0xffff_ff80_0000_0000;
/// The size of kernel address space.
pub const KERNEL_ASPACE_SIZE: usize = 0x0000_0010_0000_0000;
/// The base of user address space.
pub const USER_ASPACE_BASE: usize = 0x0000_0001_0000_0000;
/// The size of user address space.
pub const USER_ASPACE_SIZE: usize = KERNEL_ASPACE_BASE - USER_ASPACE_BASE;
