#![no_std]
#![no_main]
#![allow(improper_ctypes)]
use module_std::InfoStruct;

#[used]
#[link_section = ".info"]
#[no_mangle]
static MODULE_INFO: InfoStruct =
    InfoStruct::with_name("hello");

extern {
    fn print(msg : &str);
}

#[no_mangle]
pub extern "C" fn init() -> usize {
    //let function = MODULE_INFO.get_func(0);
    //function("test");
    unsafe {
        print("test");
    }
    0
}
