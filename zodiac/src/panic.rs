use core::panic::PanicInfo;

use alloc::format;

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    unsafe extern "Rust" {
        fn __zodiac_panic_handler(info: &PanicInfo) -> !;
    }

    unsafe {
        __zodiac_panic_handler(info);
    }
}
