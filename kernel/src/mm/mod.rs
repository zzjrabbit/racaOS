use good_memory_allocator::SpinLockedAllocator;

mod physical;
mod vm;

pub use physical::*;
pub use vm::*;

#[global_allocator]
static ALLOCATOR: SpinLockedAllocator = SpinLockedAllocator::empty();

#[allow(static_mut_refs)]
pub fn init() {
    const KERNEL_HEAP_SIZE: usize = 4 * 1024 * 1024; //1 MB

    static mut KERNEL_HEAP: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

    unsafe {
        ALLOCATOR.init(KERNEL_HEAP.as_ptr() as usize, KERNEL_HEAP_SIZE);
    }
}

pub fn page_count(size: usize) -> usize {
    (size + 4095) / 4096
}
