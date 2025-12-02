#![no_std]

use core::panic::PanicInfo;

pub use mostd_core::*;
pub use mostd_macros::*;
pub use zodiac_dylib::*;

#[zodiac_dylib::panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    unsafe {
        unsafe extern "Rust" {
            fn kernel_panic_handler(info: &PanicInfo) -> !;
        }

        kernel_panic_handler(info);
    }
}
