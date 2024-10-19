#![allow(improper_ctypes_definitions)]

pub extern "C" fn print(msg: &str) -> usize {
    crate::print!("{}", msg);
    0
}
