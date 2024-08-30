use alloc::{string::String, vec::Vec};
use raca_std::{
    fs::{change_cwd, FileDescriptor, FileType, OpenMode},
    println,
};

pub fn cd(args: Vec<String>) {
    if args.len() != 2 {
        println!("Usage: cd <folder>\n");
        return;
    }

    let path = args[1].clone();

    let k = FileDescriptor::open(path.as_str(), OpenMode::Read);
    if let Err(_) = k {
        println!("cd: {}: No such directory", path);
        return;
    }

    if k.unwrap().get_type() == FileType::File {
        println!("cd: {}: No such directory", path);
        return;
    }

    k.unwrap().close();

    change_cwd(path.clone());
}
