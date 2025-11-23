use limine::request::{HhdmRequest, MemoryMapRequest};
use loongarch64::{PhysAddr, VirtAddr};
use spin::{Lazy, Mutex};

pub use heap::Allocator;

use crate::mem::frame::BitmapFrameAllocator;

mod frame;
mod heap;
mod paging;

pub(crate) fn init() {
    paging::init();
    heap::init();
}

#[used]
#[unsafe(link_section = ".requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

static PHYSICAL_MEMORY_OFFSET: Lazy<usize> =
    Lazy::new(|| HHDM_REQUEST.get_response().unwrap().offset() as usize);

static FRAME_ALLOCATOR: Lazy<Mutex<BitmapFrameAllocator>> = Lazy::new(|| {
    let memory_map = MEMORY_MAP_REQUEST.get_response().unwrap();
    Mutex::new(BitmapFrameAllocator::init(memory_map))
});

pub fn convert_physical_to_virtual(physical_address: PhysAddr) -> VirtAddr {
    VirtAddr::new((physical_address + *PHYSICAL_MEMORY_OFFSET as u64).as_u64())
}

pub fn convert_virtual_to_physical(virtual_address: VirtAddr) -> PhysAddr {
    PhysAddr::new(virtual_address.as_u64() - *PHYSICAL_MEMORY_OFFSET as u64)
}
