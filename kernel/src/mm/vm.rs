use alloc::{sync::Arc, vec::Vec};
use core::range::Range;
use spin::{Lazy, Mutex};
use x86_64::{
    PhysAddr, VirtAddr,
    registers::control::Cr3,
    structures::paging::{
        Mapper, OffsetPageTable, Page, PageSize, PageTableFlags, PhysFrame, Size4KiB, Translate,
    },
};

use crate::{
    error::{RcError, RcResult},
    hal::{ExtendedPageTable, convert_physical_to_virtual, ref_current_page_table},
};

use super::{PhysicalMemory, page_count};

pub const KERNEL_ASPACE_BASE: u64 = 0xffff_ff80_0000_0000;
pub const KERNEL_ASPACE_SIZE: u64 = 0x0000_0080_0000_0000;
pub const USER_ASPACE_BASE: u64 = 0x0000_0000_0100_0000;
pub const USER_ASPACE_SIZE: u64 = KERNEL_ASPACE_BASE - USER_ASPACE_BASE;

pub static KERNEL_VIRTUAL_MEMORY: Lazy<Arc<VirtualMemory>> =
    Lazy::new(|| VirtualMemory::new_kernel());

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
            Arc::new(Mutex::new(unsafe {
                KERNEL_VIRTUAL_MEMORY.page_table().lock().deep_copy()
            })),
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

    pub fn is_empty(&self) -> bool {
        self.page_count == 0
    }

    pub fn start_address(&self) -> usize {
        self.start_address
    }

    pub fn create_child(&self, page_range: Range<usize>) -> Arc<Self> {
        let start_page = page_range.start;
        let end_page = page_range.end;

        let child_start_address = self.start_address + 4096 * start_page;
        let page_count = end_page - start_page;

        let child = Self::new(child_start_address, page_count, self.page_table.clone());
        self.inner.lock().children.push(child.clone());
        child
    }

    pub fn create_child_absolute(&self, start_address: usize, page_count: usize) -> Arc<Self> {
        let child = Self::new(start_address, page_count, self.page_table.clone());
        self.inner.lock().children.push(child.clone());
        child
    }

    pub fn get_absolute(&self, start_address: usize, page_count: usize) -> Arc<Self> {
        Self::new(start_address, page_count, self.page_table.clone())
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
            if id + 1 < inner.children.len() {
                let next = &inner.children[id + 1];

                if next.start_address < child.len() + child.start_address {
                    continue;
                }
                if next.start_address - child.start_address - child.len() >= size {
                    start_address = Ok(child.start_address + child.len());
                }
            } else {
                let current = &inner.children[id];
                let last_address = current.start_address + current.page_count * 4096;
                let end_address = self.start_address + self.page_count * 4096;

                if end_address - last_address >= size {
                    start_address = Ok(last_address);
                }
            }
        }

        if inner.children.is_empty() && self.page_count >= page_count {
            start_address = Ok(self.start_address);
        }

        let child = Self::new(start_address?, page_count, self.page_table.clone());

        inner.children.push(child.clone());

        Ok(child)
    }

    pub fn load_page_table(&self) {
        let page_table = self.page_table.lock();
        let frame = PhysFrame::containing_address(page_table.physical_address());

        let flags = Cr3::read().1;
        unsafe {
            Cr3::write(frame, flags);
        }
    }

    pub unsafe fn read(&self, offset: usize, buff: &mut [u8]) {
        for (sub_offset, byte) in buff.iter_mut().enumerate() {
            let address =
                VirtAddr::new(self.start_address as u64 + offset as u64 + sub_offset as u64);
            let physical_address = self
                .page_table
                .lock()
                .translate_addr(address)
                .expect("Failed to translate address!");
            let virtual_address = convert_physical_to_virtual(physical_address).as_u64();
            unsafe {
                *byte = (virtual_address as *mut u8).read();
            }
        }
    }

    pub unsafe fn write(&self, offset: usize, buffer: &[u8]) {
        let mut written: usize = 0;

        while written < buffer.len() {
            let current_address = self.start_address as u64 + offset as u64 + written as u64;
            let page_offset = current_address % Size4KiB::SIZE;
            let remaining = (buffer.len() - written) as u64;
            let chunk_size = (Size4KiB::SIZE - page_offset).min(remaining) as usize;

            let physical_address = self
                .page_table()
                .lock()
                .translate_addr(VirtAddr::new(current_address))
                .expect("Failed to translate address!");
            let virtual_address = convert_physical_to_virtual(physical_address);

            unsafe {
                core::ptr::copy_nonoverlapping(
                    buffer[written..written + chunk_size].as_ptr(),
                    virtual_address.as_mut_ptr::<u8>(),
                    chunk_size,
                );
            }
            written += chunk_size;
        }
    }

    pub fn page_table(&self) -> Arc<Mutex<OffsetPageTable<'static>>> {
        self.page_table.clone()
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
        let mut flag = PageTableFlags::PRESENT;

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
        let mut ret = Ok(());

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
                let map_result = self.virtual_memory.page_table.lock().map_to(
                    virtual_page,
                    physical_frame,
                    self.permissions.to_flags(),
                    &mut *frame_allocator,
                );
                if let Ok(flusher) = map_result {
                    flusher.flush();
                } else {
                    ret = Err(RcError::FailedToMap);
                }
            }
        }
        ret
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
