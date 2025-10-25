use bitflags::bitflags;
use ostd::{
    mm::{CachePolicy, PageFlags, PageProperty, Vaddr},
    task::Task,
};

use {
    crate::{AsThread, UserThreadData},
    memory::{align_down_by_page_size, align_up_by_page_size},
};

use super::*;

bitflags! {
    #[derive(Debug)]
    pub struct MMapProtection: i32 {
        const READ = 1;
        const WRITE = 2;
        const EXECUTE = 4;
    }

    #[derive(Debug)]
    pub struct MMapFlags: i32 {
        const SHARED = 1;
        const PRIVATE = 2;
        const FIXED = 0x10;
        const ANONYMOUS = 0x20;
        const NORESERVE = 0x4000;
        const POPULATE = 0x8000;
    }
}

impl MMapProtection {
    pub fn to_mmu_flags(&self) -> PageFlags {
        let mut flags = PageFlags::empty();

        if self.contains(MMapProtection::READ) {
            flags |= PageFlags::R;
        }
        if self.contains(MMapProtection::WRITE) {
            flags |= PageFlags::W;
        }
        if self.contains(MMapProtection::EXECUTE) {
            flags |= PageFlags::X;
        }

        flags
    }
}

pub fn mmap(
    address: Vaddr,
    len: usize,
    protection: MMapProtection,
    flags: MMapFlags,
    _fd: usize,
    _offset: usize,
) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();
    let memory_info = data.memory_info();

    let protection_flags = protection.to_mmu_flags();

    let address = if address == 0 {
        memory_info.allocate(len)
    } else {
        memory_info.allocate_at(address, len, flags.contains(MMapFlags::FIXED))
    }?
    .start_address();
    log::trace!(
        "mmap: address: {:x} len: {:x} prot: {:?} flags: {:?}",
        address,
        len,
        protection_flags,
        flags
    );

    memory_info.vmar().map(
        address,
        len,
        PageProperty::new_user(protection_flags, CachePolicy::Writeback),
        true,
    )?;

    Ok(address as isize)
}

pub fn mprotect(address: Vaddr, len: usize, protection: MMapProtection) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();
    let memory_info = data.memory_info();

    let protection_flags = protection.to_mmu_flags();

    log::trace!(
        "mprotect address: {:x} prot: {:?}",
        address,
        protection_flags
    );

    memory_info.vmar().protect(address, len, protection_flags)?;

    Ok(0)
}

pub fn munmap(address: Vaddr, len: usize) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    let aligned_address = align_down_by_page_size(address);
    let len = align_up_by_page_size(address + len - aligned_address);

    data.memory_info().vmar().unmap(address, len)?;

    Ok(0)
}
