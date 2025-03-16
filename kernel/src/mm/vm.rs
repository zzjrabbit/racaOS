use alloc::{sync::Arc, vec::Vec};
use core::range::Range;
use spin::Mutex;
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{Mapper, OffsetPageTable, Page, PageTableFlags, PhysFrame, Size4KiB},
};

use crate::{
    error::{RcError, RcResult},
    hal::{ExtendedPageTable, ref_current_page_table},
};

use super::{PhysicalMemory, page_count};

pub const KERNEL_ASPACE_BASE: u64 = 0xffff_ff80_0000_0000;
pub const KERNEL_ASPACE_SIZE: u64 = 0x0000_0080_0000_0000;
pub const USER_ASPACE_BASE: u64 = 0x0000_0000_0100_0000;
pub const USER_ASPACE_SIZE: u64 = KERNEL_ASPACE_BASE - USER_ASPACE_BASE;

crate::kernel_object! {
    pub struct VirtualMemory {
        start_address: usize = start_address,
        page_count: usize = page_count,
        inner: Mutex<VirtualMemoryInner> = Mutex::new(Default::default()),
        page_table: Arc<Mutex<OffsetPageTable<'static>>> = page_table,
    }

    fn new(start_address: usize, page_count: usize, page_table: Arc<Mutex<OffsetPageTable<'static>>>) {}
}

impl VirtualMemory {
    pub fn new_root() -> Arc<Self> {
        Self::new(
            USER_ASPACE_BASE as usize,
            page_count(USER_ASPACE_SIZE as usize),
            Arc::new(Mutex::new(unsafe { ref_current_page_table().deep_copy() })),
        )
    }

    pub fn new_kernel() -> Arc<Self> {
        Self::new(
            KERNEL_ASPACE_BASE as usize,
            page_count(KERNEL_ASPACE_SIZE as usize),
            Arc::new(Mutex::new(ref_current_page_table())),
        )
    }
}

impl VirtualMemory {
    pub fn as_ptr(&self) -> *const u8 {
        self.start_address as *const u8
    }
    
    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.start_address as *mut u8
    }
    
    /// return how many BYTES are there in this VirtualMemory Object
    pub fn len(&self) -> usize {
        self.page_count * 4096
    }

    pub fn create_child(&self, page_range: Range<usize>) -> Arc<Self> {
        let start_page = page_range.start;
        let end_page = page_range.end;

        let child_start_address = self.start_address + start_page + 4096 * start_page;
        let page_count = end_page - start_page;

        let child = Self::new(child_start_address, page_count, self.page_table.clone());
        self.inner.lock().children.push(child.clone());
        child
    }

    pub fn allocate_child(&self, page_count: usize) -> RcResult<Arc<Self>> {
        let size = page_count * 4096;

        self.inner
            .lock()
            .children
            .sort_by(|vm1, vm2| vm1.start_address.cmp(&vm2.start_address));

        let mut start_address = Err(RcError::AllocationFailed);

        let mut inner = self.inner.lock();

        for (id, child) in inner.children.iter().enumerate() {
            let next = &inner.children[id + 1];

            if next.start_address - child.start_address - child.len() >= size {
                start_address = Ok(child.start_address + child.len());
            }
        }

        if inner.children.len() == 0 && self.page_count >= page_count {
            start_address = Ok(self.start_address);
        }

        let child = Self::new(start_address?, page_count, self.page_table.clone());

        inner.children.push(child.clone());

        Ok(child)
    }
}

#[derive(Default)]
struct VirtualMemoryInner {
    children: Vec<Arc<VirtualMemory>>,
}

bitflags::bitflags! {
    /// Generic memory flags.
    #[derive(Clone, Copy, Debug)]
    pub struct MMUFlags: usize {
        const READ      = 1 << 2;
        const WRITE     = 1 << 3;
        const EXECUTE   = 1 << 4;
        const USER      = 1 << 5;
        const RXW = Self::READ.bits() | Self::WRITE.bits() | Self::EXECUTE.bits();
    }
}

impl MMUFlags {
    pub fn to_flags(&self) -> PageTableFlags {
        let mut flag = PageTableFlags::empty();

        if self.contains(Self::READ) {
            flag |= PageTableFlags::PRESENT;
        }

        if self.contains(Self::WRITE) {
            flag |= PageTableFlags::WRITABLE;
        }

        if !self.contains(Self::EXECUTE) {
            flag |= PageTableFlags::NO_EXECUTE;
        }

        if self.contains(Self::USER) {
            flag |= PageTableFlags::USER_ACCESSIBLE;
        }

        flag
    }
}

pub struct VmMapping {
    /// The permission limitation of the vmar
    permissions: MMUFlags,
    physical_memory: Arc<PhysicalMemory>,
    virtual_memory: Arc<VirtualMemory>,
}

impl VmMapping {
    pub fn new(
        permissions: MMUFlags,
        virtual_memory: Arc<VirtualMemory>,
        physical_memory: Arc<PhysicalMemory>,
    ) -> Self {
        Self {
            permissions,
            virtual_memory,
            physical_memory,
        }
    }
}

impl VmMapping {
    pub fn map(self: &Arc<Self>) -> RcResult<()> {
        let mut frame_allocator = crate::hal::FRAME_ALLOCATOR.lock();

        for frame in 0..self.physical_memory.frame_count() as u64 {
            let physical_frame = PhysFrame::<Size4KiB>::from_start_address(PhysAddr::new(
                self.physical_memory.start_address() as u64 + frame * 4096,
            ))
            .unwrap();
            let virtual_page = Page::from_start_address(VirtAddr::new(
                self.virtual_memory.start_address as u64 + frame * 4096,
            ))
            .unwrap();

            unsafe {
                self.virtual_memory
                    .page_table
                    .lock()
                    .map_to(
                        virtual_page,
                        physical_frame,
                        self.permissions.to_flags(),
                        &mut *frame_allocator,
                    )
                    .map_err(|_| RcError::FailedToMap)?
                    .flush();
            }
        }
        Ok(())
    }

    pub fn unmap(&self) -> RcResult<()> {
        for page in 0..self.virtual_memory.page_count as u64 {
            let page = Page::<Size4KiB>::from_start_address(VirtAddr::new(
                self.virtual_memory.start_address as u64 + page * 4096,
            ))
            .unwrap();

            self.virtual_memory
                .page_table
                .lock()
                .unmap(page)
                .map_err(|_| RcError::FailedToUnmap)?
                .1
                .flush();
        }
        Ok(())
    }
}
