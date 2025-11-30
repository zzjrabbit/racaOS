#![no_std]
#![forbid(unsafe_code)]

extern crate memory;

use memory::tester;
use mostd::{entry, println};

#[entry(hello)]
fn main() {
    println!("HELLO!");
    tester();
}
