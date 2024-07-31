use alloc::{string::String, vec::Vec};
use raca_std::fs::FileDescriptor;
use core::fmt::Write;

pub fn mount(stdio: &mut FileDescriptor, args: Vec<String>) {
    if args.len() != 3 {
        writeln!(stdio, "Usage: mount <path> <partition>\n").unwrap();
        return;
    }

    let path = args[2].clone();
    let partition = args[1].clone();

    raca_std::fs::mount(path.clone(), partition.clone()).unwrap_or_else(|_| {
        writeln!(stdio, "Failed to mount {} to {}\n", path, partition).unwrap();
    });
}

