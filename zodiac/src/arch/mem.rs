use alloc::{sync::Arc, vec::Vec};
use loongarch64::{
    PhysAddr, PrivilegeLevel, VirtAddr,
    registers::{PgdHigh, PgdLow},
    structures::paging::{
        CachePolicy, FrameAllocator, Mapper, OffsetPageTable, Page, PageProperty, PageSize,
        PageTable, PageTableFlags, PhysFrame, Size1GiB, Size2MiB, Size4KiB, Translate,
        TranslateResult,
    },
};
use spin::RwLock;

use crate::{
    MapError, QueryError, UnmapError, UpdateError, ZodiacError,
    mem::{
        BitmapFrameAllocator, FRAME_ALLOCATOR, GeneralPageTable, MMUFlags, PhysicalAddress,
        Privilege, VirtualAddress, convert_physical_to_virtual, convert_virtual_to_physical,
    },
};

fn zodiac_property_converter(property: crate::mem::PageProperty) -> PageProperty {
    let mut result = PageProperty::new();

    let flags = property.flags;
    if !flags.contains(MMUFlags::READ) {
        result.add_flags(PageTableFlags::NO_READ);
    }
    if flags.contains(MMUFlags::WRITE) {
        result.add_flags(PageTableFlags::WRITABLE);
        result.add_flags(PageTableFlags::DIRTY);
    }
    if !flags.contains(MMUFlags::EXECUTE) {
        result.add_flags(PageTableFlags::NO_EXECUTE);
    }
    if flags.contains(MMUFlags::HUGE_PAGE) {
        result.add_flags(PageTableFlags::HUGE_PAGE);
        result.add_flags(PageTableFlags::GLOBAL_FOR_HUGE_PAGE);
    } else {
        result.add_flags(PageTableFlags::GLOBAL);
    }

    match property.privilege {
        Privilege::User => {
            result.set_privilege(PrivilegeLevel::Privilege3);
            result.set_privilege_restriction(false);
        }
        Privilege::KernelOnly => {
            result.set_privilege(PrivilegeLevel::Privilege0);
            result.set_privilege_restriction(true);
        }
        Privilege::UserOnly => {
            result.set_privilege(PrivilegeLevel::Privilege3);
            result.set_privilege_restriction(true);
        }
    }

    match property.cache_policy {
        crate::mem::CachePolicy::CacheCoherent => {
            result.set_cache_policy(CachePolicy::CoherentCached);
        }
        crate::mem::CachePolicy::WeaklyOrderedUnCached => {
            result.set_cache_policy(CachePolicy::WeaklyOrderedUnCached);
        }
        crate::mem::CachePolicy::StronglyOrderedUnCached => {
            result.set_cache_policy(CachePolicy::StronglyOrderedUnCached);
        }
    }

    result
}

fn loongarch64_property_converter(
    property: PageProperty,
    huge_page: bool,
) -> crate::mem::PageProperty {
    let mut flags = MMUFlags::empty();

    let source_flags = property.flags();
    if !source_flags.contains(PageTableFlags::NO_READ) {
        flags |= MMUFlags::READ;
    }
    if source_flags.contains(PageTableFlags::DIRTY) {
        flags |= MMUFlags::WRITE;
    }
    if !source_flags.contains(PageTableFlags::NO_EXECUTE) {
        flags |= MMUFlags::EXECUTE;
    }
    if huge_page {
        flags |= MMUFlags::HUGE_PAGE;
    }

    let cache_policy = match property.cache_policy() {
        CachePolicy::CoherentCached => crate::mem::CachePolicy::CacheCoherent,
        CachePolicy::WeaklyOrderedUnCached => crate::mem::CachePolicy::WeaklyOrderedUnCached,
        CachePolicy::StronglyOrderedUnCached => crate::mem::CachePolicy::StronglyOrderedUnCached,
        CachePolicy::Reserved => unreachable!(),
    };
    let privilege = {
        let privilege_level = property.privilege();
        let privilege_restriction = property.privilege_restriction();
        match privilege_level {
            PrivilegeLevel::Privilege0 => Privilege::KernelOnly,
            PrivilegeLevel::Privilege3 => {
                if privilege_restriction {
                    Privilege::UserOnly
                } else {
                    Privilege::User
                }
            }
            _ => unreachable!(),
        }
    };

    crate::mem::PageProperty::new(flags, cache_policy, privilege)
}

impl GeneralPageTable for OffsetPageTable<'_> {
    fn map(
        &mut self,
        page: crate::mem::Page,
        paddr: crate::mem::PhysicalAddress,
        property: crate::mem::PageProperty,
    ) -> Result<(), crate::ZodiacError> {
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
                        zodiac_property_converter(property),
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
        use loongarch64::structures::paging::Mapper;
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
    ) -> Result<
        (
            PhysicalAddress,
            crate::mem::PageProperty,
            crate::mem::PageSize,
        ),
        ZodiacError,
    > {
        match self.translate(VirtAddr::new(vaddr as u64)) {
            TranslateResult::Mapped {
                frame,
                offset,
                property,
            } => {
                let address = frame.start_address().as_u64() as PhysicalAddress;
                let property =
                    loongarch64_property_converter(property, frame.size() != Size4KiB::SIZE);

                let size = match frame.size() {
                    Size4KiB::SIZE => crate::mem::PageSize::Size4K,
                    Size2MiB::SIZE => crate::mem::PageSize::Size2M,
                    Size1GiB::SIZE => crate::mem::PageSize::Size1G,
                    _ => unreachable!(),
                };

                Ok((address + offset as usize, property, size))
            }
            TranslateResult::NotMapped => Err(QueryError::NotMappedYet.into()),
            TranslateResult::InvalidFrameAddress(_) => Err(QueryError::InvalidFrameAddress.into()),
        }
    }

    fn update(
        &mut self,
        vaddr: VirtualAddress,
        updater: fn(&mut crate::mem::PageProperty),
    ) -> Result<crate::mem::PageSize, ZodiacError> {
        let Ok((_, _, page_size)) = self.query(vaddr) else {
            return Err(UpdateError::NotMappedYet.into());
        };

        let vaddr = VirtAddr::new(page_size.align_down(vaddr) as u64);
        let property = {
            let mut property = match self.translate(vaddr) {
                TranslateResult::Mapped {
                    property, frame, ..
                } => loongarch64_property_converter(property, frame.size() != Size4KiB::SIZE),
                _ => return Err(UpdateError::NotMappedYet.into()),
            };

            updater(&mut property);
            zodiac_property_converter(property)
        };

        unsafe {
            match page_size {
                crate::mem::PageSize::Size4K => self
                    .update_property(Page::<Size4KiB>::containing_address(vaddr), property)
                    .map_err(|_| UpdateError::NotMappedYet)?
                    .flush(),
                crate::mem::PageSize::Size2M => self
                    .update_property(Page::<Size2MiB>::containing_address(vaddr), property)
                    .map_err(|_| UpdateError::NotMappedYet)?
                    .flush(),
                crate::mem::PageSize::Size1G => self
                    .update_property(Page::<Size1GiB>::containing_address(vaddr), property)
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
            self.lower_half_page_table() as *const _,
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
            let kernel_page_table_ptr =
                VirtAddr::new(self.higher_half_page_table() as *const _ as u64).as_mut_ptr();
            OffsetPageTable::new(
                &mut *kernel_page_table_ptr,
                root_table,
                convert_physical_to_virtual(0) as u64,
            )
        };
        Arc::new(RwLock::new(page_table))
    }

    fn activate(&self) {
        let lower_half =
            convert_virtual_to_physical(self.lower_half_page_table() as *const _ as VirtualAddress);
        PgdLow.write(lower_half as u64);

        let higher_half = convert_virtual_to_physical(
            self.higher_half_page_table() as *const _ as VirtualAddress
        );
        PgdHigh.write(higher_half as u64);
    }
}
