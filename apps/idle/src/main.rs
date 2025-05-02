#![no_std]
#![no_main]

#[unsafe(no_mangle)]
pub extern "sysv64" fn _start() {
    common_std::dummy();
    loop {}
}
