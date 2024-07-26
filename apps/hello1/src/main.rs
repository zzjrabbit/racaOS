#![no_std]
#![no_main]
#![feature(naked_functions)]

//#[panic_handler]
//fn panic(info: &core::panic::PanicInfo) -> ! {
//    raca_std::println!("User Panic:{}", info);
//    loop {}
//}

#[no_mangle]
pub fn main() {
    loop {
    }
}
