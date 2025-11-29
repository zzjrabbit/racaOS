#![no_std]

use core::panic::PanicInfo;

pub use mostd_macros::*;

#[doc(hidden)]
pub struct ModuleInfo {
    pub name: &'static str,
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    unsafe {
        unsafe extern "Rust" {
            fn kernel_panic_handler(info: &PanicInfo) -> !;
        }

        kernel_panic_handler(info);
    }
}
