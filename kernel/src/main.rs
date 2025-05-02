#![no_std]
#![no_main]

extern crate alloc;

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    kernel::init();

    kernel::println!("test start");

    x86_64::instructions::interrupts::enable();

    loop {
        x86_64::instructions::hlt();
    }
}
