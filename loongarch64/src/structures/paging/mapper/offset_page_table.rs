use bit_field::BitField;

use crate::{
    VirtAddr,
    structures::paging::{
        CleanUp, FlagUpdateError, FrameAllocator, FrameDeallocator, FrameError, MapToError,
        MappedFrame, Mapper, MapperFlush, Page, PageProperty, PageRangeInclusive, PageTable,
        PageTableEntry, PageTableFlags, PageTableLevel, PhysFrame, Size1GiB, Size2MiB, Size4KiB,
        Translate, TranslateError, TranslateResult, UnmapError,
    },
};

#[derive(Debug)]
pub struct OffsetPageTable<'a> {
    lower_half: &'a mut PageTable,
    higher_half: &'a mut PageTable,
    offset: u64,
    page_table_walker: PageTableWalker,
}

impl<'a> OffsetPageTable<'a> {
    #[inline]
    pub unsafe fn new(
        lower_half: &'a mut PageTable,
        higher_half: &'a mut PageTable,
        offset: u64,
    ) -> Self {
        OffsetPageTable {
            lower_half,
            higher_half,
            offset,
            page_table_walker: PageTableWalker { offset },
        }
    }

    pub fn lower_half_page_table(&self) -> &PageTable {
        self.lower_half
    }

    pub fn lower_half_page_table_mut(&mut self) -> &mut PageTable {
        self.lower_half
    }

    pub fn higher_half_page_table(&self) -> &PageTable {
        self.higher_half
    }

    pub fn higher_half_page_table_mut(&mut self) -> &mut PageTable {
        self.higher_half
    }
}

impl<'a> OffsetPageTable<'a> {
    pub fn offset(&self) -> u64 {
        self.offset
    }
}

macro_rules! get_root_page_table {
    ($s: expr, $page: expr) => {
        if $page.start_address().as_u64().get_bit(63) {
            &mut $s.higher_half
        } else {
            &mut $s.lower_half
        }
    };
    (immut $s: expr, $page: expr) => {
        if $page.start_address().as_u64().get_bit(63) {
            &$s.higher_half
        } else {
            &$s.lower_half
        }
    };

    ($s: expr, addr $addr: expr) => {
        if $addr.get_bit(63) {
            &mut $s.higher_half
        } else {
            &mut $s.lower_half
        }
    };
    (immut $s: expr, addr $addr: expr) => {
        if $addr.as_u64().get_bit(63) {
            &$s.higher_half
        } else {
            &$s.lower_half
        }
    };
}

impl<'a> OffsetPageTable<'a> {
    /// Helper function for implementing Mapper. Safe to limit the scope of unsafe, see
    /// https://github.com/rust-lang/rfcs/pull/2585.
    fn map_to_1gib<A>(
        &mut self,
        page: Page<Size1GiB>,
        frame: PhysFrame<Size1GiB>,
        page_property: PageProperty,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size1GiB>, MapToError<Size1GiB>>
    where
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        let p4 = get_root_page_table!(self, page);
        let p3 = self
            .page_table_walker
            .create_next_table(&mut p4[page.p4_index()], allocator)?;

        if !p3[page.p3_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(frame));
        }
        p3[page.p3_index()].set_addr(
            frame.start_address(),
            page_property.flags() | PageTableFlags::HUGE_PAGE,
        );
        p3[page.p3_index()].set_privilege(page_property.privilege());
        p3[page.p3_index()].set_privilege_restriction(page_property.privilege_restriction());
        p3[page.p3_index()].set_cache_policy(page_property.cache_policy());

        Ok(MapperFlush::new(page))
    }

    /// Helper function for implementing Mapper. Safe to limit the scope of unsafe, see
    /// https://github.com/rust-lang/rfcs/pull/2585.
    fn map_to_2mib<A>(
        &mut self,
        page: Page<Size2MiB>,
        frame: PhysFrame<Size2MiB>,
        page_property: PageProperty,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size2MiB>, MapToError<Size2MiB>>
    where
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        let p4 = get_root_page_table!(self, page);
        let p3 = self
            .page_table_walker
            .create_next_table(&mut p4[page.p4_index()], allocator)?;
        let p2 = self
            .page_table_walker
            .create_next_table(&mut p3[page.p3_index()], allocator)?;

        if !p2[page.p2_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(frame));
        }
        p2[page.p2_index()].set_addr(frame.start_address(), page_property.flags());
        p2[page.p2_index()].set_privilege(page_property.privilege());
        p2[page.p2_index()].set_privilege_restriction(page_property.privilege_restriction());
        p2[page.p2_index()].set_cache_policy(page_property.cache_policy());

        Ok(MapperFlush::new(page))
    }

    /// Helper function for implementing Mapper. Safe to limit the scope of unsafe, see
    /// https://github.com/rust-lang/rfcs/pull/2585.
    fn map_to_4kib<A>(
        &mut self,
        page: Page<Size4KiB>,
        frame: PhysFrame<Size4KiB>,
        page_property: PageProperty,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>>
    where
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        let p4 = get_root_page_table!(self, page);
        let p3 = self
            .page_table_walker
            .create_next_table(&mut p4[page.p4_index()], allocator)?;
        let p2 = self
            .page_table_walker
            .create_next_table(&mut p3[page.p3_index()], allocator)?;
        let p1 = self
            .page_table_walker
            .create_next_table(&mut p2[page.p2_index()], allocator)?;

        if !p1[page.p1_index()].is_unused() {
            return Err(MapToError::PageAlreadyMapped(frame));
        }
        p1[page.p1_index()].set_frame(frame, page_property.flags());
        p1[page.p1_index()].set_privilege(page_property.privilege());
        p1[page.p1_index()].set_privilege_restriction(page_property.privilege_restriction());
        p1[page.p1_index()].set_cache_policy(page_property.cache_policy());

        Ok(MapperFlush::new(page))
    }
}

impl<'a> Mapper<Size1GiB> for OffsetPageTable<'a> {
    unsafe fn map_to<A>(
        &mut self,
        page: Page<Size1GiB>,
        frame: PhysFrame<Size1GiB>,
        page_property: PageProperty,
        frame_allocator: &mut A,
    ) -> Result<MapperFlush<Size1GiB>, MapToError<Size1GiB>>
    where
        Self: Sized,
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        self.map_to_1gib(page, frame, page_property, frame_allocator)
    }

    fn unmap(
        &mut self,
        page: Page<Size1GiB>,
    ) -> Result<(PhysFrame<Size1GiB>, MapperFlush<Size1GiB>), UnmapError> {
        let p4 = get_root_page_table!(self, page);
        let p3 = self
            .page_table_walker
            .next_table_mut(&mut p4[page.p4_index()])?;

        let p3_entry = &mut p3[page.p3_index()];
        let flags = p3_entry.flags();

        if !flags.contains(PageTableFlags::PRESENT) {
            return Err(UnmapError::PageNotMapped);
        }
        if !flags.contains(PageTableFlags::HUGE_PAGE) {
            return Err(UnmapError::ParentEntryHugePage);
        }

        let frame = PhysFrame::from_start_address(p3_entry.addr())
            .map_err(|()| UnmapError::InvalidFrameAddress(p3_entry.addr()))?;

        p3_entry.set_unused();
        Ok((frame, MapperFlush::new(page)))
    }

    unsafe fn update_property(
        &mut self,
        page: Page<Size1GiB>,
        mut property: PageProperty,
    ) -> Result<MapperFlush<Size1GiB>, FlagUpdateError> {
        let p4 = get_root_page_table!(self, page);
        let p3 = self
            .page_table_walker
            .next_table_mut(&mut p4[page.p4_index()])?;

        if p3[page.p3_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        property.add_flags(PageTableFlags::HUGE_PAGE);

        p3[page.p3_index()].set_property(property);

        Ok(MapperFlush::new(page))
    }

    fn translate_page(&self, page: Page<Size1GiB>) -> Result<PhysFrame<Size1GiB>, TranslateError> {
        let p4 = get_root_page_table!(immut self, page);
        let p3 = self.page_table_walker.next_table(&p4[page.p4_index()])?;

        let p3_entry = &p3[page.p3_index()];

        if p3_entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        PhysFrame::from_start_address(p3_entry.addr())
            .map_err(|()| TranslateError::InvalidFrameAddress(p3_entry.addr()))
    }
}

impl Mapper<Size2MiB> for OffsetPageTable<'_> {
    #[inline]
    unsafe fn map_to<A>(
        &mut self,
        page: Page<Size2MiB>,
        frame: PhysFrame<Size2MiB>,
        page_property: PageProperty,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size2MiB>, MapToError<Size2MiB>>
    where
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        self.map_to_2mib(page, frame, page_property, allocator)
    }

    fn unmap(
        &mut self,
        page: Page<Size2MiB>,
    ) -> Result<(PhysFrame<Size2MiB>, MapperFlush<Size2MiB>), UnmapError> {
        let p4 = get_root_page_table!(self, page);
        let p3 = self
            .page_table_walker
            .next_table_mut(&mut p4[page.p4_index()])?;
        let p2 = self
            .page_table_walker
            .next_table_mut(&mut p3[page.p3_index()])?;

        let p2_entry = &mut p2[page.p2_index()];
        let flags = p2_entry.flags();

        if !flags.contains(PageTableFlags::PRESENT) {
            return Err(UnmapError::PageNotMapped);
        }
        if !flags.contains(PageTableFlags::HUGE_PAGE) {
            return Err(UnmapError::ParentEntryHugePage);
        }

        let frame = PhysFrame::from_start_address(p2_entry.addr())
            .map_err(|()| UnmapError::InvalidFrameAddress(p2_entry.addr()))?;

        p2_entry.set_unused();
        Ok((frame, MapperFlush::new(page)))
    }

    unsafe fn update_property(
        &mut self,
        page: Page<Size2MiB>,
        mut property: PageProperty,
    ) -> Result<MapperFlush<Size2MiB>, FlagUpdateError> {
        let p4 = get_root_page_table!(self, page);
        let p3 = self
            .page_table_walker
            .next_table_mut(&mut p4[page.p4_index()])?;
        let p2 = self
            .page_table_walker
            .next_table_mut(&mut p3[page.p3_index()])?;

        if p2[page.p2_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        property.add_flags(PageTableFlags::HUGE_PAGE);

        p2[page.p2_index()].set_property(property);

        Ok(MapperFlush::new(page))
    }

    fn translate_page(&self, page: Page<Size2MiB>) -> Result<PhysFrame<Size2MiB>, TranslateError> {
        let p4 = get_root_page_table!(immut self, page);
        let p3 = self.page_table_walker.next_table(&p4[page.p4_index()])?;
        let p2 = self.page_table_walker.next_table(&p3[page.p3_index()])?;

        let p2_entry = &p2[page.p2_index()];

        if p2_entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        PhysFrame::from_start_address(p2_entry.addr())
            .map_err(|()| TranslateError::InvalidFrameAddress(p2_entry.addr()))
    }
}

impl Mapper<Size4KiB> for OffsetPageTable<'_> {
    #[inline]
    unsafe fn map_to<A>(
        &mut self,
        page: Page<Size4KiB>,
        frame: PhysFrame<Size4KiB>,
        page_property: PageProperty,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>>
    where
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        self.map_to_4kib(page, frame, page_property, allocator)
    }

    fn unmap(
        &mut self,
        page: Page<Size4KiB>,
    ) -> Result<(PhysFrame<Size4KiB>, MapperFlush<Size4KiB>), UnmapError> {
        let p4 = get_root_page_table!(self, page);
        let p3 = self
            .page_table_walker
            .next_table_mut(&mut p4[page.p4_index()])?;
        let p2 = self
            .page_table_walker
            .next_table_mut(&mut p3[page.p3_index()])?;
        let p1 = self
            .page_table_walker
            .next_table_mut(&mut p2[page.p2_index()])?;

        let p1_entry = &mut p1[page.p1_index()];

        let frame = p1_entry.frame();

        p1_entry.set_unused();
        Ok((frame, MapperFlush::new(page)))
    }

    unsafe fn update_property(
        &mut self,
        page: Page<Size4KiB>,
        property: PageProperty,
    ) -> Result<MapperFlush<Size4KiB>, FlagUpdateError> {
        let p4 = get_root_page_table!(self, page);
        let p3 = self
            .page_table_walker
            .next_table_mut(&mut p4[page.p4_index()])?;
        let p2 = self
            .page_table_walker
            .next_table_mut(&mut p3[page.p3_index()])?;
        let p1 = self
            .page_table_walker
            .next_table_mut(&mut p2[page.p2_index()])?;

        if p1[page.p1_index()].is_unused() {
            return Err(FlagUpdateError::PageNotMapped);
        }

        p1[page.p1_index()].set_property(property);

        Ok(MapperFlush::new(page))
    }

    fn translate_page(&self, page: Page<Size4KiB>) -> Result<PhysFrame<Size4KiB>, TranslateError> {
        let p4 = get_root_page_table!(immut self, page);
        let p3 = self.page_table_walker.next_table(&p4[page.p4_index()])?;
        let p2 = self.page_table_walker.next_table(&p3[page.p3_index()])?;
        let p1 = self.page_table_walker.next_table(&p2[page.p2_index()])?;

        let p1_entry = &p1[page.p1_index()];

        if p1_entry.is_unused() {
            return Err(TranslateError::PageNotMapped);
        }

        PhysFrame::from_start_address(p1_entry.addr())
            .map_err(|()| TranslateError::InvalidFrameAddress(p1_entry.addr()))
    }
}

#[derive(Debug)]
struct PageTableWalker {
    offset: u64,
}

impl PageTableWalker {
    fn frame_to_pointer(&self, frame: PhysFrame) -> *mut PageTable {
        let page_table_ptr = VirtAddr::new((frame.start_address() + self.offset).into());
        page_table_ptr.as_mut_ptr()
    }

    #[inline]
    fn next_table<'b>(
        &self,
        entry: &'b PageTableEntry,
    ) -> Result<&'b PageTable, PageTableWalkError> {
        let page_table_ptr = self.frame_to_pointer(entry.frame());
        let page_table: &PageTable = unsafe { &*page_table_ptr };

        Ok(page_table)
    }

    #[inline]
    fn next_table_mut<'b>(
        &self,
        entry: &'b mut PageTableEntry,
    ) -> Result<&'b mut PageTable, PageTableWalkError> {
        let page_table_ptr = self.frame_to_pointer(entry.frame());
        let page_table: &mut PageTable = unsafe { &mut *page_table_ptr };

        Ok(page_table)
    }

    fn create_next_table<'b, A>(
        &self,
        entry: &'b mut PageTableEntry,
        allocator: &mut A,
    ) -> Result<&'b mut PageTable, PageTableCreateError>
    where
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        let created;

        if entry.is_unused() {
            if let Some(frame) = allocator.allocate_frame() {
                entry.set_frame(frame, PageTableFlags::empty());
                created = true;
            } else {
                return Err(PageTableCreateError::FrameAllocationFailed);
            }
        } else {
            created = false;
        }

        let page_table = match self.next_table_mut(entry) {
            Err(PageTableWalkError::MappedToHugePage) => {
                return Err(PageTableCreateError::MappedToHugePage);
            }
            Err(PageTableWalkError::NotMapped) => panic!("entry should be mapped at this point"),
            Ok(page_table) => page_table,
        };

        if created {
            page_table.zero();
        }
        Ok(page_table)
    }
}

impl CleanUp for OffsetPageTable<'_> {
    #[inline]
    unsafe fn clean_up<D>(&mut self, frame_deallocator: &mut D)
    where
        D: FrameDeallocator<Size4KiB>,
    {
        unsafe {
            self.clean_up_addr_range(
                PageRangeInclusive {
                    start: Page::from_start_address(VirtAddr::new(0)).unwrap(),
                    end: Page::from_start_address(VirtAddr::new(0xffff_ffff_ffff_f000)).unwrap(),
                },
                frame_deallocator,
            )
        }
    }

    unsafe fn clean_up_addr_range<D>(
        &mut self,
        range: PageRangeInclusive,
        frame_deallocator: &mut D,
    ) where
        D: FrameDeallocator<Size4KiB>,
    {
        unsafe fn clean_up(
            page_table: &mut PageTable,
            page_table_walker: &PageTableWalker,
            level: PageTableLevel,
            range: PageRangeInclusive,
            frame_deallocator: &mut impl FrameDeallocator<Size4KiB>,
        ) -> bool {
            if range.is_empty() {
                return false;
            }

            let table_addr = range
                .start
                .start_address()
                .align_down(level.table_address_space_alignment());

            let start = range.start.page_table_index(level);
            let end = range.end.page_table_index(level);

            if let Some(next_level) = level.next_lower_level() {
                let offset_per_entry = level.entry_address_space_alignment();
                for (i, entry) in page_table
                    .iter_mut()
                    .enumerate()
                    .take(usize::from(end) + 1)
                    .skip(usize::from(start))
                {
                    if let Ok(page_table) = page_table_walker.next_table_mut(entry) {
                        let start = VirtAddr::forward_checked_impl(
                            table_addr,
                            (offset_per_entry as usize) * i,
                        )
                        .unwrap();
                        let end = start + (offset_per_entry - 1);
                        let start = Page::<Size4KiB>::containing_address(start);
                        let start = start.max(range.start);
                        let end = Page::<Size4KiB>::containing_address(end);
                        let end = end.min(range.end);
                        unsafe {
                            if clean_up(
                                page_table,
                                page_table_walker,
                                next_level,
                                Page::range_inclusive(start, end),
                                frame_deallocator,
                            ) {
                                let frame = entry.frame();
                                entry.set_unused();
                                frame_deallocator.deallocate_frame(frame);
                            }
                        }
                    }
                }
            }

            page_table.iter().all(PageTableEntry::is_unused)
        }

        unsafe {
            clean_up(
                self.lower_half,
                &self.page_table_walker,
                PageTableLevel::Four,
                range,
                frame_deallocator,
            );

            clean_up(
                self.higher_half,
                &self.page_table_walker,
                PageTableLevel::Four,
                range,
                frame_deallocator,
            );
        }
    }
}

impl Translate for OffsetPageTable<'_> {
    #[allow(clippy::inconsistent_digit_grouping)]
    fn translate(&self, addr: VirtAddr) -> TranslateResult {
        let p4 = get_root_page_table!(immut self, addr addr);
        let p3 = match self.page_table_walker.next_table(&p4[addr.p4_index()]) {
            Ok(page_table) => page_table,
            Err(PageTableWalkError::NotMapped) => return TranslateResult::NotMapped,
            Err(PageTableWalkError::MappedToHugePage) => {
                panic!("level 4 entry has huge page bit set")
            }
        };
        let p2 = match self.page_table_walker.next_table(&p3[addr.p3_index()]) {
            Ok(page_table) => page_table,
            Err(PageTableWalkError::NotMapped) => return TranslateResult::NotMapped,
            Err(PageTableWalkError::MappedToHugePage) => {
                let entry = &p3[addr.p3_index()];
                let frame = PhysFrame::containing_address(entry.addr());
                #[allow(clippy::unusual_byte_groupings)]
                let offset = addr.as_u64() & 0o_777_777_7777;
                let property = entry.property();

                return TranslateResult::Mapped {
                    frame: MappedFrame::Size1GiB(frame),
                    offset,
                    property,
                };
            }
        };
        let p1 = match self.page_table_walker.next_table(&p2[addr.p2_index()]) {
            Ok(page_table) => page_table,
            Err(PageTableWalkError::NotMapped) => return TranslateResult::NotMapped,
            Err(PageTableWalkError::MappedToHugePage) => {
                let entry = &p2[addr.p2_index()];
                let frame = PhysFrame::containing_address(entry.addr());
                #[allow(clippy::unusual_byte_groupings)]
                let offset = addr.as_u64() & 0o_777_7777;
                let property = entry.property();

                return TranslateResult::Mapped {
                    frame: MappedFrame::Size2MiB(frame),
                    offset,
                    property,
                };
            }
        };

        let p1_entry = &p1[addr.p1_index()];

        if p1_entry.is_unused() {
            return TranslateResult::NotMapped;
        }

        let frame = match PhysFrame::from_start_address(p1_entry.addr()) {
            Ok(frame) => frame,
            Err(()) => return TranslateResult::InvalidFrameAddress(p1_entry.addr()),
        };
        let offset = u64::from(addr.page_offset());
        let property = p1_entry.property();
        TranslateResult::Mapped {
            frame: MappedFrame::Size4KiB(frame),
            offset,
            property,
        }
    }
}

#[derive(Debug)]
enum PageTableWalkError {
    NotMapped,
    MappedToHugePage,
}

#[derive(Debug)]
enum PageTableCreateError {
    MappedToHugePage,
    FrameAllocationFailed,
}

impl From<PageTableCreateError> for MapToError<Size4KiB> {
    #[inline]
    fn from(err: PageTableCreateError) -> Self {
        match err {
            PageTableCreateError::MappedToHugePage => MapToError::ParentEntryHugePage,
            PageTableCreateError::FrameAllocationFailed => MapToError::FrameAllocationFailed,
        }
    }
}

impl From<PageTableCreateError> for MapToError<Size2MiB> {
    #[inline]
    fn from(err: PageTableCreateError) -> Self {
        match err {
            PageTableCreateError::MappedToHugePage => MapToError::ParentEntryHugePage,
            PageTableCreateError::FrameAllocationFailed => MapToError::FrameAllocationFailed,
        }
    }
}

impl From<PageTableCreateError> for MapToError<Size1GiB> {
    #[inline]
    fn from(err: PageTableCreateError) -> Self {
        match err {
            PageTableCreateError::MappedToHugePage => MapToError::ParentEntryHugePage,
            PageTableCreateError::FrameAllocationFailed => MapToError::FrameAllocationFailed,
        }
    }
}

impl From<FrameError> for PageTableWalkError {
    #[inline]
    fn from(err: FrameError) -> Self {
        match err {
            FrameError::HugeFrame => PageTableWalkError::MappedToHugePage,
            FrameError::FrameNotPresent => PageTableWalkError::NotMapped,
        }
    }
}

impl From<PageTableWalkError> for UnmapError {
    #[inline]
    fn from(err: PageTableWalkError) -> Self {
        match err {
            PageTableWalkError::MappedToHugePage => UnmapError::ParentEntryHugePage,
            PageTableWalkError::NotMapped => UnmapError::PageNotMapped,
        }
    }
}

impl From<PageTableWalkError> for FlagUpdateError {
    #[inline]
    fn from(err: PageTableWalkError) -> Self {
        match err {
            PageTableWalkError::MappedToHugePage => FlagUpdateError::ParentEntryHugePage,
            PageTableWalkError::NotMapped => FlagUpdateError::PageNotMapped,
        }
    }
}

impl From<PageTableWalkError> for TranslateError {
    #[inline]
    fn from(err: PageTableWalkError) -> Self {
        match err {
            PageTableWalkError::MappedToHugePage => TranslateError::ParentEntryHugePage,
            PageTableWalkError::NotMapped => TranslateError::PageNotMapped,
        }
    }
}
