#![no_std]
#![feature(new_range_api)]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![feature(get_mut_unchecked)]
// #![recursion_limit = "512"]
// #![allow(clippy::cast_possible_truncation)]

extern crate alloc;

pub mod error;
pub mod hal;
pub mod ipc;
pub mod logging;
pub mod mm;
pub mod object;
pub mod signal;
pub mod syscall;
pub mod task;

use limine::{BaseRevision, request::StackSizeRequest};

#[used]
#[unsafe(link_section = ".requests")]
pub static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[unsafe(link_section = ".requests")]
pub static STACK_SIZE: StackSizeRequest = StackSizeRequest::new().with_size(256 * 1024);

pub fn init() {
    logging::init();
    log::info!("INIT Done: logging");

    mm::init();
    log::info!("INIT Done: mm");
    hal::init();
    log::info!("INIT Done: hal");
    task::scheduler::init();
    log::info!("INIT Done: scheduler");

    log::info!("racaOS initialized.");
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("Panic: {}", info);
    loop {
        x86_64::instructions::hlt();
    }
}
