use std::ffi::*;

use libc::*;

fn main() {
    unsafe {
        let file = open(c"/input.txt".as_ptr(), O_RDONLY);
        let mut buffer = [0u8; 32];
        let bytes_read = read(file, buffer.as_mut_ptr() as *mut c_void, buffer.len());

        if bytes_read < 0 {
            panic!("read failed.");
        }

        println!(
            "input.txt: {}",
            core::str::from_utf8(&buffer[..bytes_read as usize]).unwrap()
        );
    }
}
