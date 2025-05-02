#![no_std]
#![no_main]

use alloc::{string::String, vec::Vec};

extern crate alloc;

#[unsafe(no_mangle)]
pub fn main(_handles: Vec<u32>) {
    loop {
        let mut command = String::new();
        std::read_line(&mut command).unwrap();
        std::print((command + "\n").as_str()).unwrap();
    }
}
