use alloc::{string::String, vec::Vec};
use raca_std::println;

pub fn echo(args: Vec<String>) {
    if args.len() < 2 {
        return;
    }

    let output = args.join(" ");

    println!("{}", output);
}
