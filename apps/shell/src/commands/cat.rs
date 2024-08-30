use alloc::{string::String, vec, vec::Vec};
use raca_std::{
    fs::{FileDescriptor, OpenMode},
    io::stdout,
    println,
};

pub fn cat(args: Vec<String>) {
    if args.len() != 2 {
        println!("Usage: cat <file>\n");
        return;
    }

    let file_path = args[1].clone();
    if let Ok(fd) = FileDescriptor::open(file_path.as_str(), OpenMode::Read) {
        let size = fd.size();
        let mut buf = vec![0; size];
        fd.read(buf.as_mut_slice());
        stdout().write(&buf);
        stdout().write(&[b'\n']);
    } else {
        stdout().write("Can't find the file.\n".as_bytes());
    }
}
