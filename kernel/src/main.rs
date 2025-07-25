#![no_std]
#![no_main]

use core::panic::PanicInfo;

use zodiac::mem::{MMUFlags, PageSize, PhysicalMemory, VirtualMemory};

#[zodiac::main]
pub fn main() {
    log::info!("Zodiac Initialize done, entering kernel.");
    
    let _root = VirtualMemory::kernel();
    let child = _root.allocate(None, 8192, 4096).unwrap();
    let phys_mem = PhysicalMemory::new(2, PageSize::Size4K, false);
    child.map(0, phys_mem, MMUFlags::READ | MMUFlags::WRITE).unwrap();
    
    let buffer = unsafe{core::slice::from_raw_parts_mut(child.start_address() as *mut u8, 8192)};
    buffer.fill(0);
    
    log::info!("test done");
    
    loop {}
}

#[zodiac::panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    log::error!("panic: {}!", info);
    loop {}
}
