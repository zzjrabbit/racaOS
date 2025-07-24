#![no_std]
#![no_main]

use core::panic::PanicInfo;

use aegis::mem::VirtualMemory;

#[aegis::main]
pub fn main() {
    log::info!("Aegis Initialize done, entering kernel.");
    
    let root = VirtualMemory::kernel();
    
    loop {}
}

#[aegis::panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    log::error!("panic: {}!", info);
    loop {}
}
