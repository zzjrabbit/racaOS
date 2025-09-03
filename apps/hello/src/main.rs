use std::{ffi::*, fs::File, io::Read};

use libc::*;

fn main() {
    let mut file = File::open("/input.txt").unwrap();

    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();

    println!("read: {:x?}", buf);

    unsafe {
        let file = open(c"/input.txt".as_ptr(), O_RDONLY);

        lseek(file, -19, SEEK_END);

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
