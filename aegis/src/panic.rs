use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    unsafe extern "Rust" {
        fn __aegis_panic_handler(info: &PanicInfo) -> !;
    }

    unsafe {
        __aegis_panic_handler(info);
    }
}
