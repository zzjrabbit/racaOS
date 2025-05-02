use spin::Lazy;
use talc::{OomHandler, Span, Talc, Talck};

use crate::memory::{MMUFlags, PhysicalMemory, VirtualMemory};

const HEAP_START: usize = 0x20000000;
const ONCE_ALLOCATION_SIZE: usize = 1 * 1024 * 1024;

#[global_allocator]
static ALLOCATOR: Talck<spin::Mutex<()>, OomHandlerImpl> =
    Talc::new(OomHandlerImpl::default()).lock();

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("Kernel heap allocation error: {:?}", layout)
}

struct OomHandlerImpl(Span);

impl OomHandlerImpl {
    const fn default() -> Self {
        OomHandlerImpl(Span::from_base_size(HEAP_START as *mut u8, 0))
    }
}

impl OomHandler for OomHandlerImpl {
    fn handle_oom(talc: &mut Talc<Self>, _layout: core::alloc::Layout) -> Result<(), ()> {
        let current_heap = talc.oom_handler.0;

        if current_heap.is_empty() {
            malloc(HEAP_START, ONCE_ALLOCATION_SIZE);
            let new_heap = Span::from_base_size(HEAP_START as *mut u8, ONCE_ALLOCATION_SIZE);
            unsafe { talc.claim(new_heap).unwrap() };
            talc.oom_handler.0 = new_heap;
        } else {
            let (_, current_end) = current_heap.get_base_acme().unwrap();
            malloc(current_end as usize, ONCE_ALLOCATION_SIZE);
            let new_heap = current_heap.extend(0, ONCE_ALLOCATION_SIZE);
            talc.oom_handler.0 = unsafe { talc.extend(current_heap, new_heap) };
        }

        Ok(())
    }
}

// Maxium heap size: 32GB
static HEAP_MEMORY: Lazy<VirtualMemory> = Lazy::new(|| {
    let root = VirtualMemory::root_virtual_memory().unwrap();
    root.create_child(
        (HEAP_START - root.start_address().unwrap()) / 4096,
        ONCE_ALLOCATION_SIZE * 8,
    )
    .unwrap()
});

fn malloc(start: usize, size: usize) {
    let page_count = (size + 4095) / 4096;

    let virtual_memory = HEAP_MEMORY
        .create_child(
            (start - HEAP_MEMORY.start_address().unwrap()) / 4096,
            page_count,
        )
        .unwrap();

    for count in 0..page_count {
        let physical_memory = PhysicalMemory::create(1).unwrap();
        let child = virtual_memory.create_child(count, 1).unwrap();
        //let _ = child.unmap();
        let _ = child.map(physical_memory, MMUFlags::READ | MMUFlags::WRITE);
        //crate::debug::debug("good\n").unwrap();
    }

    //crate::debug::debug("good\n").unwrap();

    unsafe {
        core::ptr::write_bytes(start as *mut u8, 0, size);
    }
}
