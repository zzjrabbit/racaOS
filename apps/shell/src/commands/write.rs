use alloc::{string::String, vec::Vec};
use raca_std::fs::{FileDescriptor, OpenMode};
use core::fmt::Write;

pub fn write(stdio: &mut FileDescriptor, args: Vec<String>) {
    if args.len() < 3 {
        writeln!(stdio, "Usage: write <file> <content>\n").unwrap();
        return;
    }

    let file_path = args[1].clone();
    let content = args[2..].join(" ");

    if let Ok(file) = FileDescriptor::open(file_path.as_str(), OpenMode::Write) {
        file.write(content.as_bytes());
    } else {
        writeln!(stdio,"Can't find {}.",file_path).unwrap();
    }
}

