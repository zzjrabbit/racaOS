#![no_std]
#![feature(new_range_api)]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![recursion_limit = "512"]

use limine::BaseRevision;

extern crate alloc;

pub mod error;
pub mod hal;
pub mod logging;
pub mod mm;
pub mod object;
pub mod task;

#[used]
#[unsafe(link_section = ".limine")]
pub static BASE_REVISION: BaseRevision = BaseRevision::with_revision(2);

pub fn init() {
    logging::init();
    log::info!("INIT Done: logging");
    mm::init();
    log::info!("INIT Done: mm");
    hal::init();
    log::info!("INIT Done: hal");

    log::info!("racaOS initialized.");

    x86_64::instructions::interrupts::enable();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("Panic: {}", info);
    loop {
        x86_64::instructions::hlt();
    }
}
