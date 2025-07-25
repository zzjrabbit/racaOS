use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    unsafe extern "Rust" {
        fn __zodiac_panic_handler(info: &PanicInfo) -> !;
    }

    unsafe {
        __zodiac_panic_handler(info);
    }
}
