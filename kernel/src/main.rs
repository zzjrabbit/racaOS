#![no_std]
#![no_main]

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    kernel::init();
    kernel::println!("test start");
    unsafe {
        core::arch::asm!("int 0");
    }
    loop {}
}
