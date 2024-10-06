#![no_std]

use module_std::InfoStruct;

#[used]
#[link_section = ".info"]
static MODULE_INFO: InfoStruct =
    InfoStruct::with_name([b'h', b'e', b'l', b'l', b'o', b' ', b' ', b' ']);

#[no_mangle]
pub extern "C" fn get_info() -> &'static InfoStruct {
    &MODULE_INFO
}

#[no_mangle]
pub extern "C" fn init() -> usize {
    MODULE_INFO.get_func(0)("Hello from kernel module!")
}
