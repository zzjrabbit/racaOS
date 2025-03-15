use x86_64::registers::control::Cr3;
use x86_64::structures::paging::FrameAllocator;
use x86_64::structures::paging::FrameDeallocator;
use x86_64::structures::paging::PhysFrame;
use x86_64::structures::paging::mapper::*;
use x86_64::structures::paging::{PageTable, PageTableFlags};
use x86_64::{PhysAddr, VirtAddr};

use super::FRAME_ALLOCATOR;
use super::{BitmapFrameAllocator, convert_physical_to_virtual};

pub enum MappingType {
    UserCode,
    KernelData,
    UserData,
}

#[rustfmt::skip]
impl MappingType {
    pub fn flags(&self) -> PageTableFlags {
        match self {
            Self::UserCode => PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::USER_ACCESSIBLE,
            Self::KernelData => PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::NO_EXECUTE,
            Self::UserData => PageTableFlags::PRESENT
                | PageTableFlags::WRITABLE
                | PageTableFlags::USER_ACCESSIBLE
                | PageTableFlags::NO_EXECUTE,
        }
    }
}

pub trait ExtendedPageTable {
    fn physical_address(&self) -> PhysAddr;
    fn write_to_mapped_address(&self, buffer: &[u8], address: VirtAddr);
    /// # Safety
    ///
    /// It is mostly safe
    unsafe fn deep_copy(&self) -> OffsetPageTable<'static>;
    /// # Safety
    ///
    /// It is mostly safe
    unsafe fn free_user_page_table(&self);
}

impl ExtendedPageTable for OffsetPageTable<'_> {
    fn physical_address(&self) -> PhysAddr {
        let virtual_address = self.level_4_table() as *const _;
        PhysAddr::new(virtual_address as u64 - self.phys_offset().as_u64())
    }

    fn write_to_mapped_address(&self, buffer: &[u8], address: VirtAddr) {
        for (offset, &byte) in buffer.iter().enumerate() {
            let address = address + offset as u64;
            let physical_address = self
                .translate_addr(address)
                .expect("Failed to translate address!");
            let virtual_address = convert_physical_to_virtual(physical_address).as_u64();
            unsafe { (virtual_address as *mut u8).write(byte) }
        }
    }

    unsafe fn deep_copy(&self) -> OffsetPageTable<'static> {
        let virtual_address = convert_physical_to_virtual(self.physical_address());
        let source_table = unsafe { &*virtual_address.as_ptr::<PageTable>() };

        let mut frame_allocator = FRAME_ALLOCATOR.lock();
        let mut new_page_table = unsafe { new_from_allocate(&mut frame_allocator) };
        let target_table = new_page_table.level_4_table_mut();

        unsafe {
            new_from_recursion(&mut frame_allocator, source_table, target_table, 4);
        }
        new_page_table
    }

    unsafe fn free_user_page_table(&self) {
        let mut frame_allocator = FRAME_ALLOCATOR.lock();
        unsafe {
            free_from_recursion(&mut frame_allocator, self.physical_address(), 4);
        }
    }
}

unsafe fn new_from_allocate(
    frame_allocator: &mut BitmapFrameAllocator,
) -> OffsetPageTable<'static> {
    let page_table_address = BitmapFrameAllocator::allocate_frame(frame_allocator)
        .expect("Failed to allocate frame for page table")
        .start_address();

    let new_page_table =
        unsafe { &mut *convert_physical_to_virtual(page_table_address).as_mut_ptr::<PageTable>() };

    let physical_memory_offset = convert_physical_to_virtual(PhysAddr::zero());
    let page_table = unsafe { OffsetPageTable::new(new_page_table, physical_memory_offset) };

    page_table
}

unsafe fn new_from_recursion(
    frame_allocator: &mut BitmapFrameAllocator,
    source_page_table: &PageTable,
    target_page_table: &mut PageTable,
    page_table_level: u8,
) {
    for (index, entry) in source_page_table.iter().enumerate() {
        if (page_table_level == 1)
            || entry.is_unused()
            || entry.flags().contains(PageTableFlags::HUGE_PAGE)
        {
            target_page_table[index].set_addr(entry.addr(), entry.flags());
            continue;
        }
        let mut new_page_table = unsafe { new_from_allocate(frame_allocator) };
        let new_page_table_address = PhysAddr::new(new_page_table.physical_address().as_u64());
        target_page_table[index].set_addr(new_page_table_address, entry.flags());

        let source_page_table_next =
            unsafe { &*convert_physical_to_virtual(entry.addr()).as_ptr() };
        let target_page_table_next = new_page_table.level_4_table_mut();

        unsafe {
            new_from_recursion(
                frame_allocator,
                source_page_table_next,
                target_page_table_next,
                page_table_level - 1,
            );
        }
    }
}

unsafe fn free_from_recursion(
    frame_allocator: &mut BitmapFrameAllocator,
    physical_address: PhysAddr,
    page_table_level: u8,
) {
    if page_table_level == 0 {
        unsafe {
            frame_allocator.deallocate_frame(PhysFrame::containing_address(physical_address));
        }
        return;
    }

    let virtual_address = convert_physical_to_virtual(physical_address);
    let page_table = unsafe { &mut *(virtual_address.as_mut_ptr::<PageTable>()) };

    for entry in page_table.iter() {
        if entry.is_unused() {
            continue;
        }

        if page_table_level == 1 || entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            if entry.flags().contains(MappingType::UserCode.flags()) {
                if let Ok(frame) = entry.frame() {
                    unsafe {
                        frame_allocator.deallocate_frame(frame);
                    }
                }
            }
        } else {
            unsafe {
                free_from_recursion(frame_allocator, entry.addr(), page_table_level - 1);
            }
        }
    }

    unsafe {
        frame_allocator.deallocate_frame(PhysFrame::containing_address(physical_address));
    }
}

pub fn ref_current_page_table() -> OffsetPageTable<'static> {
    let physical_address = Cr3::read().0.start_address();
    let page_table = convert_physical_to_virtual(physical_address).as_mut_ptr::<PageTable>();
    let physical_memory_offset = convert_physical_to_virtual(PhysAddr::zero());
    unsafe { OffsetPageTable::new(&mut *page_table, physical_memory_offset) }
}
