mod frame;
mod page_table;

pub use frame::*;
pub use page_table::*;

use limine::request::{HhdmRequest, MemoryMapRequest};
use spin::{Lazy, Mutex};
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{FrameDeallocator, PhysFrame},
};

static HHDM_REQUEST: HhdmRequest = HhdmRequest::with_revision(2);
static MMAP_REQUEST: MemoryMapRequest = MemoryMapRequest::with_revision(2);

static PHYSICAL_OFFSET: Lazy<u64> = Lazy::new(|| {
    let hhdm_response = HHDM_REQUEST.get_response().unwrap();
    hhdm_response.offset()
});

pub static FRAME_ALLOCATOR: Lazy<Mutex<BitmapFrameAllocator>> = Lazy::new(|| {
    Mutex::new(BitmapFrameAllocator::init(
        MMAP_REQUEST.get_response().unwrap(),
    ))
});

pub fn convert_physical_to_virtual(physical_address: PhysAddr) -> VirtAddr {
    VirtAddr::new(physical_address.as_u64() + *PHYSICAL_OFFSET)
}

pub fn convert_virtual_to_physical(virtual_address: VirtAddr) -> PhysAddr {
    PhysAddr::new(virtual_address.as_u64() - *PHYSICAL_OFFSET)
}

pub fn alloc_frames(count: usize) -> Option<usize> {
    FRAME_ALLOCATOR
        .lock()
        .allocate_frames(count)
        .map(|frame| frame.start_address().as_u64() as usize)
}

pub fn deallocate_frames(start_address: u64, count: usize) {
    for index in 0..count {
        let start_address = start_address + index as u64 * 4096;
        let frame = PhysFrame::containing_address(PhysAddr::new(start_address));
        unsafe {
            FRAME_ALLOCATOR.lock().deallocate_frame(frame);
        }
    }
}
