use limine::request::{HhdmRequest, MemoryMapRequest};
use spin::{Lazy, Mutex};

mod frame;
mod heap;
mod mmio;
mod paging;
mod physical;
mod r#virtual;

pub(crate) use frame::BitmapFrameAllocator;
pub use heap::Allocator;
pub use mmio::*;
pub(crate) use paging::{GeneralPageTable, Page};
pub use paging::{MMUFlags, PageSize};
pub use physical::*;
pub use r#virtual::*;

#[cfg(feature = "default_allocator")]
pub use heap::DefaultAllocator;

pub fn init() {
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

pub(crate) static FRAME_ALLOCATOR: Lazy<Mutex<BitmapFrameAllocator>> = Lazy::new(|| {
    let memory_map = MEMORY_MAP_REQUEST.get_response().unwrap();
    Mutex::new(BitmapFrameAllocator::init(memory_map))
});

pub type VirtualAddress = usize;
pub type PhysicalAddress = usize;

pub fn convert_physical_to_virtual(physical: PhysicalAddress) -> VirtualAddress {
    physical + *PHYSICAL_MEMORY_OFFSET
}

pub fn convert_virtual_to_physical(r#virtual: VirtualAddress) -> PhysicalAddress {
    r#virtual - *PHYSICAL_MEMORY_OFFSET
}
