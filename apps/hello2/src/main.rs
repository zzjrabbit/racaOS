#![no_std]
#![no_main]

use raca_std::print;

#[no_mangle]
pub fn main() {
    print!("Hello 2!");
    loop {
        print!("Hello 2!");
    }
}
