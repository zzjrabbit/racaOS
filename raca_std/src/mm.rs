use core::alloc::{GlobalAlloc, Layout};

fn malloc(layout: Layout) -> Result<u64, ()> {
    const MALLOC_SYSCALL_ID: u64 = 7;
    let addr = crate::syscall(MALLOC_SYSCALL_ID, layout.size(), layout.align(), 0, 0, 0);

    if addr == 0 {
        Err(())
    } else {
        Ok(addr as u64)
    }
}

fn free(addr: u64, layout: Layout) {
    const FREE_SYSCALL_ID: u64 = 8;
    crate::syscall(
        FREE_SYSCALL_ID,
        addr as usize,
        layout.size(),
        layout.align(),
        0,
        0,
    );
}

struct MemoryAllocator;

unsafe impl GlobalAlloc for MemoryAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        malloc(layout).unwrap() as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        free(ptr as u64, layout)
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: MemoryAllocator = MemoryAllocator;
