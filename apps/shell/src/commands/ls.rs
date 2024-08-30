use alloc::{string::String, vec::Vec};
use raca_std::{
    fs::{get_cwd, FileInfo, FileType},
    print, println,
};

pub fn ls(args: Vec<String>) {
    if args.len() > 2 {
        println!("Usage: ls <folder>\n");
        return;
    }

    let folder = if args.len() == 2 {
        args[1].clone()
    } else {
        get_cwd()
    };
    let infos = FileInfo::list(folder.clone());

    if infos.len() == 0 {
        println!("ls: {}: No such directory", folder);
        return;
    }

    for info in infos.iter() {
        match info.ty {
            FileType::Dir => print!("\x1b[42m{}\x1b[0m ", info.name),
            FileType::File => print!("\x1b[32m{}\x1b[0m ", info.name),
        }
    }
    println!();
}
