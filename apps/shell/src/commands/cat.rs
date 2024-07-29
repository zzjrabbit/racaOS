use core::fmt::Write;
use alloc::{string::String, vec::Vec, vec};
use raca_std::fs::{FileDescriptor, OpenMode};

pub fn cat(stdio: &mut FileDescriptor, args: Vec<String>) {
    if args.len() != 2 {
        writeln!(stdio,"Usage: cat <file>\n").unwrap();
        return;
    }
    
    let file_path = args[1].clone();
    if let Ok(fd) = FileDescriptor::open(file_path.as_str(), OpenMode::Read) {
        let size = fd.size();
        let mut buf = vec![0; size];
        fd.read(buf.as_mut_slice());
        stdio.write(&buf);
        stdio.write(&[b'\n']);
    } else {
        stdio.write("Can't find the file.\n".as_bytes());
    }
}
