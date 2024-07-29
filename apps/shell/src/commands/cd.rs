use alloc::{string::String, vec::Vec};
use raca_std::fs::{change_cwd, get_cwd, FileDescriptor};
use core::fmt::Write;

pub fn cd(stdio: &mut FileDescriptor, args: Vec<String>) {
    if args.len() != 2 {
        writeln!(stdio, "Usage: cd <folder>\n").unwrap();
        return;
    }

    let old = get_cwd();

    let path = args[1].clone();
    change_cwd(path.clone());
    if get_cwd() == old {
        writeln!(stdio, "cd: {}: No such file or directory", path).unwrap();
    }
}

