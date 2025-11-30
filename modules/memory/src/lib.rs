#![no_std]

use mostd::entry;

pub fn tester() {
    panic!("test passed!");
}

#[entry(memory)]
fn main() {}
