use alloc::{sync::Arc, vec::Vec};
use spin::{Lazy, RwLock};
use x86_64::{
    PhysAddr, VirtAddr,
    registers::control::Cr3,
    structures::paging::{
        FrameAllocator, FrameDeallocator, Mapper, OffsetPageTable, Page, PageSize, PageTable,
        PageTableFlags, PhysFrame, Size1GiB, Size2MiB, Size4KiB, Translate,
        mapper::{MapToError, TranslateResult},
    },
};

use crate::mem::{
    BitmapFrameAllocator, FRAME_ALLOCATOR, GeneralPageTable, MMUFlags, PhysicalAddress,
    VirtualAddress, convert_physical_to_virtual, convert_virtual_to_physical,
};
use crate::{MapError, QueryError, UnmapError, UpdateError, ZodiacError};

static KERNEL_PAGE_TABLE: Lazy<Arc<RwLock<dyn GeneralPageTable>>> =
    Lazy::new(|| current_page_table());

fn current_page_table() -> Arc<RwLock<dyn GeneralPageTable>> {
    let physical_address = Cr3::read().0.start_address();
    log::trace!(
        "Current page table physical address: {:?}",
        physical_address
    );

    let page_table =
        convert_physical_to_virtual(physical_address.as_u64() as PhysicalAddress) as *mut PageTable;
    let physical_memory_offset = VirtAddr::new(convert_physical_to_virtual(0) as u64);
    let page_table = unsafe { OffsetPageTable::new(&mut *page_table, physical_memory_offset) };
    Arc::new(RwLock::new(page_table))
}

pub fn kernel_page_table() -> Arc<RwLock<dyn GeneralPageTable>> {
    KERNEL_PAGE_TABLE.clone()
}

fn mmu_flags_to_page_table_flags(mmu_flags: MMUFlags) -> PageTableFlags {
    let mut result = PageTableFlags::empty();
    if mmu_flags.contains(MMUFlags::READ) {
        result |= PageTableFlags::PRESENT;
    }
    if mmu_flags.contains(MMUFlags::WRITE) {
        result |= PageTableFlags::WRITABLE;
    }
    if !mmu_flags.contains(MMUFlags::EXECUTE) {
        result |= PageTableFlags::NO_EXECUTE
    }
    if mmu_flags.contains(MMUFlags::USER) {
        result |= PageTableFlags::USER_ACCESSIBLE;
    }
    if mmu_flags.contains(MMUFlags::HUGE_PAGE) {
        result |= PageTableFlags::HUGE_PAGE;
    }
    result
}

fn page_table_flags_to_mmu_flags(flags: PageTableFlags) -> MMUFlags {
    let mut result = MMUFlags::empty();
    if flags.contains(PageTableFlags::PRESENT) {
        result |= MMUFlags::READ;
    }
    if flags.contains(PageTableFlags::WRITABLE) {
        result |= MMUFlags::WRITE;
    }
    if !flags.contains(PageTableFlags::NO_EXECUTE) {
        result |= MMUFlags::EXECUTE;
    }
    if flags.contains(PageTableFlags::USER_ACCESSIBLE) {
        result |= MMUFlags::USER;
    }
    if flags.contains(PageTableFlags::HUGE_PAGE) {
        result |= MMUFlags::HUGE_PAGE;
    }
    result
}

unsafe impl<S: PageSize> FrameAllocator<S> for BitmapFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<S>> {
        self.allocate_frames(S::SIZE as usize / 4096, S::SIZE as usize / 4096)
            .map(|address| PhysFrame::containing_address(PhysAddr::new(address as u64)))
    }
}

impl<S: PageSize> FrameDeallocator<S> for BitmapFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<S>) {
        self.deallocate_frames(
            frame.start_address().as_u64() as PhysicalAddress,
            S::SIZE as usize / 4096,
        );
    }
}

impl<S: PageSize> From<MapToError<S>> for MapError {
    fn from(value: MapToError<S>) -> Self {
        match value {
            MapToError::ParentEntryHugePage => MapError::ParentEntryHugePage,
            MapToError::FrameAllocationFailed => MapError::FrameAllocationFailed,
            MapToError::PageAlreadyMapped(_) => MapError::PageAlreadyMapped,
        }
    }
}

impl GeneralPageTable for OffsetPageTable<'_> {
    fn physical_address(&self) -> PhysicalAddress {
        let virtual_address = self.level_4_table() as *const _ as VirtualAddress;
        convert_virtual_to_physical(virtual_address)
    }

    fn map(
        &mut self,
        page: crate::mem::Page,
        paddr: PhysicalAddress,
        flags: crate::mem::MMUFlags,
    ) -> Result<(), ZodiacError> {
        let vaddr = VirtAddr::new(page.vaddr as u64);
        let paddr = PhysAddr::new(paddr as u64);

        macro_rules! map_with_size {
            ($size: ident) => {{
                let page = Page::<$size>::containing_address(vaddr);
                let frame = PhysFrame::<$size>::containing_address(paddr);
                unsafe {
                    self.map_to(
                        page,
                        frame,
                        mmu_flags_to_page_table_flags(flags),
                        &mut *FRAME_ALLOCATOR.lock(),
                    )
                    .map_err(|err| MapError::from(err))?
                    .flush();
                }
                Ok(())
            }};
        }

        match page.size {
            crate::mem::PageSize::Size4K => map_with_size!(Size4KiB),
            crate::mem::PageSize::Size2M => map_with_size!(Size2MiB),
            crate::mem::PageSize::Size1G => map_with_size!(Size1GiB),
        }
    }

    fn unmap(
        &mut self,
        vaddr: VirtualAddress,
    ) -> Result<(PhysicalAddress, crate::mem::PageSize), ZodiacError> {
        use x86_64::structures::paging::Mapper;
        match self.translate(VirtAddr::new(vaddr as u64)) {
            TranslateResult::Mapped { frame, .. } => {
                let size = crate::mem::PageSize::try_from(frame.size() as usize).unwrap();
                let address = frame.start_address().as_u64() as PhysicalAddress;

                let vaddr = VirtAddr::new(vaddr as u64);

                match frame.size() {
                    Size4KiB::SIZE => {
                        Mapper::unmap(self, Page::<Size4KiB>::containing_address(vaddr))
                            .unwrap()
                            .1
                            .flush()
                    }
                    Size2MiB::SIZE => {
                        Mapper::unmap(self, Page::<Size2MiB>::containing_address(vaddr))
                            .unwrap()
                            .1
                            .flush()
                    }
                    Size1GiB::SIZE => {
                        Mapper::unmap(self, Page::<Size1GiB>::containing_address(vaddr))
                            .unwrap()
                            .1
                            .flush()
                    }
                    _ => unreachable!(),
                }

                Ok((address, size))
            }
            TranslateResult::NotMapped => Err(UnmapError::NotMappedYet.into()),
            TranslateResult::InvalidFrameAddress(_) => Err(UnmapError::InvalidFrameAddress.into()),
        }
    }

    fn query(
        &mut self,
        vaddr: VirtualAddress,
    ) -> Result<(PhysicalAddress, MMUFlags, crate::mem::PageSize), ZodiacError> {
        match self.translate(VirtAddr::new(vaddr as u64)) {
            TranslateResult::Mapped {
                frame,
                offset,
                flags,
            } => {
                let address = frame.start_address().as_u64() as PhysicalAddress;
                let flags = page_table_flags_to_mmu_flags(flags);

                let size = match frame.size() {
                    Size4KiB::SIZE => crate::mem::PageSize::Size4K,
                    Size2MiB::SIZE => crate::mem::PageSize::Size2M,
                    Size1GiB::SIZE => crate::mem::PageSize::Size1G,
                    _ => unreachable!(),
                };

                Ok((address + offset as usize, flags, size))
            }
            TranslateResult::NotMapped => Err(QueryError::NotMappedYet.into()),
            TranslateResult::InvalidFrameAddress(_) => Err(QueryError::InvalidFrameAddress.into()),
        }
    }

    fn update(
        &mut self,
        vaddr: VirtualAddress,
        flags: MMUFlags,
    ) -> Result<crate::mem::PageSize, ZodiacError> {
        let Ok((_, _, page_size)) = self.query(vaddr) else {
            return Err(UpdateError::NotMappedYet.into());
        };

        let vaddr = VirtAddr::new(page_size.align_down(vaddr) as u64);
        let flags = mmu_flags_to_page_table_flags(flags);

        unsafe {
            match page_size {
                crate::mem::PageSize::Size4K => self
                    .update_flags(Page::<Size4KiB>::containing_address(vaddr), flags)
                    .map_err(|_| UpdateError::NotMappedYet)?
                    .flush(),
                crate::mem::PageSize::Size2M => self
                    .update_flags(Page::<Size2MiB>::containing_address(vaddr), flags)
                    .map_err(|_| UpdateError::NotMappedYet)?
                    .flush(),
                crate::mem::PageSize::Size1G => self
                    .update_flags(Page::<Size1GiB>::containing_address(vaddr), flags)
                    .map_err(|_| UpdateError::NotMappedYet)?
                    .flush(),
            }
        }
        Ok(page_size)
    }

    fn deep_copy(&self, remove_write: bool) -> Arc<RwLock<dyn GeneralPageTable>> {
        let frame_allocator = &mut FRAME_ALLOCATOR.lock();

        let root_table_frame =
            <BitmapFrameAllocator as FrameAllocator<Size4KiB>>::allocate_frame(frame_allocator)
                .expect("Failed to allocate frame for root page table")
                .start_address();

        let target_root_vaddr = VirtAddr::new(convert_physical_to_virtual(
            root_table_frame.as_u64() as PhysicalAddress
        ) as u64);
        let root_table: &mut PageTable = unsafe { &mut *target_root_vaddr.as_mut_ptr() };
        root_table.zero();

        let mut stack: Vec<(*const PageTable, *mut PageTable, u8)> = alloc::vec![(
            convert_physical_to_virtual(self.physical_address()) as *const _,
            root_table as *mut _,
            4
        )];

        while let Some((source_table, target_table, level)) = stack.pop() {
            for (index, entry) in (unsafe { &*source_table })
                .iter()
                .enumerate()
                .filter(|(_, entry)| !entry.is_unused())
            {
                if level == 1 || entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                    let mut flags = entry.flags();
                    if remove_write {
                        flags.remove(PageTableFlags::WRITABLE);
                    }

                    unsafe {
                        (&mut *target_table)[index].set_addr(entry.addr(), flags);
                    }
                } else {
                    let target_child_frame =
                        <BitmapFrameAllocator as FrameAllocator<Size4KiB>>::allocate_frame(
                            frame_allocator,
                        )
                        .expect("Failed to allocate frame for child page table")
                        .start_address();

                    let target_child_vaddr = VirtAddr::new(convert_physical_to_virtual(
                        target_child_frame.as_u64() as PhysicalAddress,
                    ) as u64);
                    let target_child_table =
                        unsafe { &mut *target_child_vaddr.as_mut_ptr::<PageTable>() };
                    target_child_table.zero();

                    unsafe {
                        (&mut *target_table)[index].set_addr(target_child_frame, entry.flags());
                    }

                    let source_child_vaddr = VirtAddr::new(convert_physical_to_virtual(
                        entry.addr().as_u64() as PhysicalAddress,
                    ) as u64);
                    stack.push((source_child_vaddr.as_ptr(), target_child_table, level - 1));
                }
            }
        }

        let page_table = unsafe {
            OffsetPageTable::new(
                root_table,
                VirtAddr::new(convert_physical_to_virtual(0) as u64),
            )
        };
        Arc::new(RwLock::new(page_table))
    }

    fn switch(&self) {
        let frame = PhysFrame::containing_address(PhysAddr::new(self.physical_address() as u64));

        let flags = Cr3::read().1;
        unsafe {
            Cr3::write(frame, flags);
        }
    }
}
