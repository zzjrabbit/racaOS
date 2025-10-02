use bitflags::bitflags;
use ostd::{
    mm::{tlb::TlbFlushOp, CachePolicy, PageFlags, PageProperty, Vaddr},
    task::{disable_preempt, Task},
};

use crate::{
    mem::{align_down_by_page_size, align_up_by_page_size},
    task::{AsThread, MemoryRegion, UserThreadData},
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
        memory_info.allocate_at(address, len)
    }?
    .start_address();
    log::trace!(
        "mmap: address: {:x} prot: {:?} flags: {:?}",
        address,
        protection_flags,
        flags
    );

    memory_info.with_unused_regions_mut(|regions| {
        regions.push((
            MemoryRegion::new(address, len),
            PageProperty::new_user(protection_flags, CachePolicy::Writeback),
        ))
    });

    Ok(address as isize)
}

pub fn mprotect(address: Vaddr, len: usize, protection: MMapProtection) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();
    let memory_info = data.memory_info();

    let protection_flags = protection.to_mmu_flags();
    let mut found = false;

    log::trace!(
        "mprotect address: {:x} prot: {:?}",
        address,
        protection_flags
    );

    memory_info.with_unused_regions_mut(|regions| {
        for (region, flags) in regions.iter_mut() {
            if region.contains(address) {
                flags.flags |= protection_flags;
                found = true;
            }
        }
    });

    if !found {
        let disable_preempt_guard = disable_preempt();

        let len = align_up_by_page_size(len);

        let vm_space = memory_info.vm_space();

        vm_space
            .cursor_mut(&disable_preempt_guard, &(address..address + len))?
            .protect_next(len, |flags, _| *flags |= protection_flags)
            .unwrap();

        let mut cursor = vm_space.cursor_mut(&disable_preempt_guard, &(address..address + len))?;
        cursor
            .flusher()
            .issue_tlb_flush(TlbFlushOp::for_range(address..address + len));
        cursor.flusher().dispatch_tlb_flush();
    }

    Ok(0)
}

pub fn munmap(address: Vaddr, len: usize) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    let aligned_address = align_down_by_page_size(address);
    let len = align_up_by_page_size(address + len - aligned_address);

    let disable_preempt_guard = disable_preempt();

    data.memory_info()
        .vm_space()
        .cursor_mut(&disable_preempt_guard, &(address..address + len))?
        .unmap(len);

    Ok(0)
}
