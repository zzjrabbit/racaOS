#![no_std]
#![no_main]

use alloc::sync::Arc;
use kernel::mm::{MMUFlags, VmMapping};

extern crate alloc;

#[unsafe(no_mangle)]
pub extern "C" fn kmain() -> ! {
    kernel::init();

    kernel::println!("test start");

    loop {}
}
