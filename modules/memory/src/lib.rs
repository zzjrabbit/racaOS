#![no_std]

use mostd::main;

pub fn tester() {
    panic!("test passed!");
}

#[main(memory)]
fn main() {}
