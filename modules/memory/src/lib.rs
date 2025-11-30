#![no_std]
#![forbid(unsafe_code)]

use mostd::entry;

pub fn tester() {
    panic!("test passed!");
}

#[entry(memory)]
fn main() {}
