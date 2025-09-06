use bitflags::bitflags;
use zodiac::{
    mem::{MMUFlags, VirtualAddress},
    task::Task,
};

use crate::task::{MemoryRegion, ThreadData};

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
    pub fn to_mmu_flags(&self) -> MMUFlags {
        let mut flags = MMUFlags::USER;

        if self.contains(MMapProtection::READ) {
            flags |= MMUFlags::READ;
        }
        if self.contains(MMapProtection::WRITE) {
            flags |= MMUFlags::WRITE;
        }
        if self.contains(MMapProtection::EXECUTE) {
            flags |= MMUFlags::EXECUTE;
        }

        flags
    }
}

pub fn mmap(
    address: VirtualAddress,
    len: usize,
    protection: MMapProtection,
    flags: MMapFlags,
    _fd: usize,
    _offset: usize,
) -> SyscallResult {
    let thread = Task::current();
    let data = thread.data().downcast_ref::<ThreadData>().unwrap();

    let protection_flags = protection.to_mmu_flags();

    let (address, _, page_size) = if address == 0 {
        data.allocate(len, false)
    } else {
        data.allocate_at(address, len, false)
    }?;
    log::trace!(
        "mmap: address: {:x} prot: {:?} flags: {:?}",
        address,
        protection_flags,
        flags
    );

    data.unused_region
        .write()
        .push((MemoryRegion::new(address, len), protection_flags, page_size));

    Ok(address as isize)
}

pub fn mprotect(address: VirtualAddress, len: usize, protection: MMapProtection) -> SyscallResult {
    let thread = Task::current();
    let data = thread.data().downcast_ref::<ThreadData>().unwrap();

    let protection_flags = protection.to_mmu_flags();
    let mut found = false;

    log::trace!(
        "mprotect address: {:x} prot: {:?}",
        address,
        protection_flags
    );

    for (region, flags, page_size) in data.unused_region.write().iter_mut() {
        if region.contains(address) {
            let page_size = *page_size;
            let _len = page_size.align_up(len);

            *flags |= protection_flags;
            found = true;
        }
    }

    if !found {
        let (_, flags, page_size) = data.vm_space.query(address)?;
        let len = page_size.align_up(len);

        data.vm_space
            .cursor(address, page_size)?
            .protect(len, flags | protection_flags)?;
    }

    Ok(0)
}

pub fn munmap(address: VirtualAddress, len: usize) -> SyscallResult {
    let thread = Task::current();
    let data = thread.data().downcast_ref::<ThreadData>().unwrap();

    let (_, _, page_size) = data.vm_space.query(address)?;

    let aligned_address = page_size.align_down(address);
    let len = page_size.align_up(address + len - aligned_address);
    let page_count = len / page_size as usize;

    for id in 0..page_count {
        let (pm, _, _) = data
            .vm_space
            .query(aligned_address + id * page_size as usize)?;
        pm.deallocate();
    }

    data.vm_space.cursor(address, page_size)?.unmap(len)?;

    Ok(0)
}
