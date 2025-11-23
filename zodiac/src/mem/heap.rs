use core::alloc::GlobalAlloc;

/// Allocator trait for custom memory allocators.
/// Enable `default_allocator` feature to use the default allocator.
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
