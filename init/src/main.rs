#![no_std]
#![no_main]

use zodiac::{main, println};

mod mem;
mod module;

extern crate alloc;

#[main]
pub fn main() {
    module::init();
    loop {}
}

#[zodiac::panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    println!("Panic occurred: {}", info);
    loop {}
}
