use limine::request::{HhdmRequest, MemoryMapRequest};
use loongarch64::registers::init_pwc;
use spin::{Lazy, Mutex};

pub(crate) use frame::BitmapFrameAllocator;
pub use heap::Allocator;
pub(crate) use page_table::GeneralPageTable;
pub use page_table::{CachePolicy, MMUFlags, Page, PageProperty, PageSize, Privilege};
pub use physical::{PhysicalMemory, PhysicalMemoryAllocOptions};
pub use vm_space::VmSpace;

mod frame;
mod heap;
mod page_table;
mod physical;
mod vm_space;

pub type PhysicalAddress = usize;
pub type VirtualAddress = usize;

pub(crate) fn init() {
    init_pwc();
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

pub fn convert_physical_to_virtual(physical_address: PhysicalAddress) -> VirtualAddress {
    physical_address + *PHYSICAL_MEMORY_OFFSET
}

pub fn convert_virtual_to_physical(virtual_address: VirtualAddress) -> PhysicalAddress {
    virtual_address - *PHYSICAL_MEMORY_OFFSET
}
