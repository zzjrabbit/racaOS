use core::alloc::GlobalAlloc;

pub trait Allocator: GlobalAlloc + Sync {
    fn init(&self);
}

unsafe extern "Rust" {
    static __ZODIAC_GLOBAL_ALLOCATOR: &'static dyn Allocator;
}

pub fn init() {
    unsafe {
        __ZODIAC_GLOBAL_ALLOCATOR.init();
    }
}

#[global_allocator]
static HEAP: Heap = Heap;

struct Heap;

unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe { __ZODIAC_GLOBAL_ALLOCATOR.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        unsafe {
            __ZODIAC_GLOBAL_ALLOCATOR.dealloc(ptr, layout);
        }
    }
}

#[cfg(feature = "default_allocator")]
pub struct DefaultAllocator(good_memory_allocator::SpinLockedAllocator);

#[cfg(feature = "default_allocator")]
impl DefaultAllocator {
    pub const fn new() -> Self {
        DefaultAllocator(good_memory_allocator::SpinLockedAllocator::empty())
    }
}

#[cfg(feature = "default_allocator")]
impl Default for DefaultAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "default_allocator")]
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

#[cfg(feature = "default_allocator")]
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
