#![no_std]

extern crate alloc;

use core::{alloc::GlobalAlloc, panic::PanicInfo};

use mostd_core::ModuleInfo;

#[used]
#[unsafe(no_mangle)]
pub static _MODULE_INFO: ModuleInfo = ModuleInfo { name: "core-dylib" };

#[unsafe(no_mangle)]
pub fn init() {}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    unsafe {
        unsafe extern "Rust" {
            fn kernel_panic_handler(info: &PanicInfo) -> !;
        }

        kernel_panic_handler(info);
    }
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        unsafe extern "Rust" {
            fn alloc(layout: core::alloc::Layout) -> *mut u8;
        }
        unsafe { alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        unsafe extern "Rust" {
            fn dealloc(ptr: *mut u8, layout: core::alloc::Layout);
        }
        unsafe { dealloc(ptr, layout) }
    }
}
