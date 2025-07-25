use core::alloc::GlobalAlloc;

use linked_list_allocator::LockedHeap;
use spin::Lazy;

struct Heap(Lazy<LockedHeap>);

impl Heap {
    #[allow(static_mut_refs)]
    const fn new() -> Self {
        Heap(Lazy::new(|| {
            unsafe { LockedHeap::new(KERNEL_HEAP.as_mut_ptr(), KERNEL_HEAP_SIZE) }
        }))
    }
    
    fn force(&self) {
        Lazy::force(&self.0);
    }
}

unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe{
            self.0.alloc(layout)
        }
    }
    
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        unsafe{
            self.0.dealloc(ptr, layout);
        }
    }
}

#[global_allocator]
static ALLOCATOR: Heap = Heap::new();

const KERNEL_HEAP_SIZE: usize = 16 * 1024 * 1024; //16 MB

static mut KERNEL_HEAP: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

#[allow(static_mut_refs)]
pub fn init() {
    
    unsafe {
        let ptr = KERNEL_HEAP.as_ptr();
        crate::println!("heap start: {:p}", ptr);
    }
    
    ALLOCATOR.force();
}
