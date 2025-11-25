#![no_std]
#![no_main]

use core::slice::from_raw_parts_mut;

use zodiac::{
    main,
    mem::{CachePolicy, MMUFlags, PageProperty, PhysicalMemoryAllocOptions, Privilege, VmSpace},
    println,
};

mod mem;

#[main]
pub fn main() {
    let addr = 0xffff_c000_0000_0000usize;
    let size = 4096usize;

    let physical_memory = PhysicalMemoryAllocOptions::new().allocate().unwrap();

    let vm_space = VmSpace::kernel();
    vm_space
        .cursor(addr)
        .unwrap()
        .map(
            &physical_memory,
            PageProperty::new(
                MMUFlags::READ | MMUFlags::WRITE,
                CachePolicy::CacheCoherent,
                Privilege::KernelOnly,
            ),
        )
        .unwrap();

    let buffer = unsafe { from_raw_parts_mut(addr as *mut u8, size) };
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
