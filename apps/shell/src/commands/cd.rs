use alloc::{string::String, vec::Vec};
use raca_std::fs::{change_cwd, FileDescriptor, FileType, OpenMode};
use core::fmt::Write;

pub fn cd(stdio: &mut FileDescriptor, args: Vec<String>) {
    if args.len() != 2 {
        writeln!(stdio, "Usage: cd <folder>\n").unwrap();
        return;
    }

    let path = args[1].clone();

    let k = FileDescriptor::open(path.as_str(), OpenMode::Read);
    if let Err(_) = k {
        writeln!(stdio, "cd: {}: No such directory", path).unwrap();
        return ;
    }

    if k.unwrap().get_type() == FileType::File {
        writeln!(stdio, "cd: {}: No such directory", path).unwrap();
        return ;
    }
    
    k.unwrap().close();

    change_cwd(path.clone());
}

