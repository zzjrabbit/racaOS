use x86_64::structures::paging::{FrameAllocator, FrameDeallocator, PhysFrame};
use x86_64::{PhysAddr, VirtAddr};
use crate::memory::FRAME_ALLOCATOR;

pub mod hpet;
pub mod keyboard;
pub mod mouse;
pub mod serial;
pub mod pci;

pub fn init() {
    hpet::init();
    keyboard::init();
    mouse::init();
}

pub fn alloc_for_dma() -> (PhysAddr,VirtAddr) {
    let phys = FRAME_ALLOCATOR.lock().allocate_frame().unwrap().start_address();
    let virt = crate::memory::convert_physical_to_virtual(phys);
    (phys, virt)
}

pub fn dealloc_for_dma(virt_addr: VirtAddr) {
    let phys = crate::memory::convert_virtual_to_physical(virt_addr);
    unsafe {
        FRAME_ALLOCATOR.lock().deallocate_frame(PhysFrame::containing_address(phys));
    }
}
