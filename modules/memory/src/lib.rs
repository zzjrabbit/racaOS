#![no_std]
#![forbid(unsafe_code)]

use mostd::entry;

mod vmar;
mod vmo;

pub fn tester() {
    panic!("test passed!");
}

#[entry(memory)]
fn main() {}
