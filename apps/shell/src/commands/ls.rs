use alloc::{string::String, vec::Vec};
use raca_std::fs::{get_cwd, FileDescriptor, FileInfo, FileType};
use core::fmt::Write;

pub fn ls(stdin: &mut FileDescriptor,args: Vec<String>) {
    if args.len() > 2 {
        writeln!(stdin, "Usage: ls <folder>\n").unwrap();
        return;
    }

    let folder = if args.len() == 2 {
        args[1].clone()
    }else {
        get_cwd()
    };
    let infos = FileInfo::list(folder.clone());
    
    if infos.len() == 0 {
        writeln!(stdin, "ls: {}: No such directory", folder).unwrap();
        return;
    }

    for info in infos.iter() {
        match info.ty {
            FileType::Dir => write!(stdin, "\x1b[42m{}\x1b[0m ",info.name).unwrap(),
            FileType::File => write!(stdin, "\x1b[32m{}\x1b[0m ",info.name).unwrap(),
        }
    }
    writeln!(stdin).unwrap();
}
