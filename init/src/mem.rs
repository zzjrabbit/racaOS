use core::alloc::GlobalAlloc;

use zodiac::{global_allocator, mem::Allocator};

#[global_allocator]
static ALLOCATOR: DefaultAllocator = DefaultAllocator::new();

/// # Default Allocator
/// This allocator manages 8MB of memory.
pub struct DefaultAllocator(good_memory_allocator::SpinLockedAllocator);

impl DefaultAllocator {
    pub const fn new() -> Self {
        DefaultAllocator(good_memory_allocator::SpinLockedAllocator::empty())
    }
}

impl Default for DefaultAllocator {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl GlobalAlloc for DefaultAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe { self.0.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        unsafe {
            self.0.dealloc(ptr, layout);
        }
    }
}

impl Allocator for DefaultAllocator {
    #[allow(static_mut_refs)]
    fn init(&self) {
        const HEAP_SIZE: usize = 8 * 1024 * 1024;
        static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

        unsafe {
            self.0.init(HEAP.as_mut_ptr() as usize, HEAP_SIZE);
        }
    }
}
