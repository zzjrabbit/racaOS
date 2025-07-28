#![no_std]
#![no_main]

use core::panic::PanicInfo;

use zodiac::mem::{MMUFlags, PageSize, PhysicalMemoryAllocOptions, VirtualMemorySpace};

mod heap;

#[zodiac::main]
pub fn main() {
    log::info!("Zodiac Initialize done, entering kernel.");

    let root = VirtualMemorySpace::new_kernel();
    let mut cursor = root.cursor(0x100000, PageSize::Size4K).unwrap();
    let phys_mem = PhysicalMemoryAllocOptions::default()
        .count(2)
        .allocate()
        .unwrap();
    cursor
        .map(&phys_mem, MMUFlags::READ | MMUFlags::WRITE)
        .unwrap();

    let buffer = unsafe { core::slice::from_raw_parts_mut(0x100000 as *mut u8, 8192) };
    buffer.fill(0);

    log::info!("test done, total time: {}s", zodiac::hal::timer::elapsed().as_secs_f64());

    loop {}
}

#[zodiac::panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    log::error!("panic: {}!", info);
    loop {}
}
