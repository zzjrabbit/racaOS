use std::{fs::File, io::Read};

fn main() {
    let mut file = File::open("/input.txt").unwrap();

    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();

    println!("read: {:x?}", buf);
}
