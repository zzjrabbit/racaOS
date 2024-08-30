use alloc::{string::String, vec::Vec};
use raca_std::{
    fs::{FileDescriptor, OpenMode},
    println,
};

pub fn write(args: Vec<String>) {
    if args.len() < 3 {
        println!("Usage: write <file> <content>\n");
        return;
    }

    let file_path = args[1].clone();
    let content = args[2..].join(" ");

    if let Ok(file) = FileDescriptor::open(file_path.as_str(), OpenMode::Write) {
        file.write(content.as_bytes());
    } else {
        println!("Can't find {}.", file_path);
    }
}
