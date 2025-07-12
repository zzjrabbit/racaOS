#![no_std]
#![no_main]

#[unsafe(no_mangle)]
pub extern "sysv64" fn _start() {
    common_std::dummy();
    loop {}
}

#[panic_handler]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    common_std::debug::debug("panic").unwrap();
    loop {}
}
