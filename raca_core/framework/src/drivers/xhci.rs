use core::num::NonZeroUsize;

use x86_64::{
    structures::paging::{Page, PageTableFlags, PhysFrame},
    PhysAddr,
};
use xhci::accessor::Mapper;
use xhci::Registers;

use crate::memory::{convert_physical_to_virtual, FRAME_ALLOCATOR, KERNEL_PAGE_TABLE};

#[derive(Clone)]
pub struct XHCIMapper;

impl Mapper for XHCIMapper {
    unsafe fn map(&mut self, phys_start: usize, bytes: usize) -> core::num::NonZeroUsize {
        let physical_address = PhysAddr::new(phys_start as u64);
        let virtual_address = convert_physical_to_virtual(physical_address);

        let pages = (bytes + 4095) / 4096;

        for page_i in 0..pages {
            use x86_64::structures::paging::Mapper;
            if let Ok(tlb) = KERNEL_PAGE_TABLE.lock().map_to(
                Page::containing_address(virtual_address + page_i as u64 * 4096),
                PhysFrame::containing_address(physical_address + page_i as u64 * 4096),
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                &mut *FRAME_ALLOCATOR.lock(),
            ) {
                tlb.flush();
            }
        }

        NonZeroUsize::new(virtual_address.as_u64() as usize).unwrap()
    }

    fn unmap(&mut self, _virt_start: usize, _bytes: usize) {}
}

pub fn get_xhci(mmio_base: usize) -> Registers<XHCIMapper> {
    unsafe { Registers::new(mmio_base, XHCIMapper) }
}
