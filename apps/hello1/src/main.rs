#![no_std]
#![no_main]
#![feature(naked_functions)]

//#[panic_handler]
//fn panic(info: &core::panic::PanicInfo) -> ! {
//    raca_std::println!("User Panic:{}", info);
//    loop {}
//}

use raca_std::{
    alloc::{string::String, vec::Vec},
    io::stdin,
    println,
    task::exit,
};

#[no_mangle]
pub fn main() -> usize {
    let mut string = String::new();
    stdin().read_line(&mut string);

    let numbers = string
        .split(" ")
        .map(|x| x.parse::<usize>().unwrap())
        .collect::<Vec<_>>();

    println!("sum: {}", numbers[0] + numbers[1]);

    exit(1);
}
