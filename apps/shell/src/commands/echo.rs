use alloc::{string::String, vec::Vec};
use raca_std::fs::FileDescriptor;
use core::fmt::Write;

pub fn echo(stdio: &mut FileDescriptor, args: Vec<String>){
    if args.len() < 2 {
        return;
    }

    let output = args.join(" ");

    writeln!(stdio, "{}", output).unwrap();
}

