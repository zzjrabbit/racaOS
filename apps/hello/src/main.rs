#![no_std]
#![no_main]

use alloc::vec::Vec;

extern crate alloc;

#[unsafe(no_mangle)]
pub fn main(_handles: Vec<u32>) {
    std::dummy();
    std::print("Hello World From Hello!\n").unwrap();

    loop {}
}
