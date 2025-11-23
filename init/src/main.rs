#![no_std]
#![no_main]

use core::slice::from_raw_parts_mut;

use zodiac::{main, println};

mod mem;

#[main]
pub fn main() {
    let buffer = unsafe { from_raw_parts_mut(0xffff_c000_0000_0000 as *mut u8, 4096) };
    buffer.fill(0x55);

    println!("Hello World! {:x?}", buffer);
    log::info!("log test");
    loop {}
}

#[zodiac::panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    println!("Panic occurred: {}", info);
    loop {}
}
