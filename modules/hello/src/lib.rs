#![no_std]

extern crate memory;

use memory::tester;
use mostd::main;

#[main(hello)]
fn main() {
    tester();
}
