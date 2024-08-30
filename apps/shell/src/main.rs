#![no_std]
#![no_main]

use alloc::{
    collections::btree_map::BTreeMap,
    format,
    string::{String, ToString},
    vec::Vec,
};
use raca_std::{fs::get_cwd, io::stdin, print, println};

extern crate alloc;

mod commands;
mod run;

fn get_prompt() -> String {
    format!(
        "\x1b[36m[\x1b[34mroot@raca \x1b[33m{}\x1b[36m]\x1b[34m:) \x1b[0m",
        get_cwd()
    )
}

type CommandFunction = fn(args: Vec<String>);

#[no_mangle]
pub fn main() -> usize {
    let mut command_function_list = BTreeMap::<&str, CommandFunction>::new();

    {
        use commands::*;
        command_function_list.insert("cat", cat);
        command_function_list.insert("cd", cd);
        command_function_list.insert("echo", echo);
        command_function_list.insert("exit", exit);
        command_function_list.insert("ls", ls);
        command_function_list.insert("mount", mount);
        command_function_list.insert("write", write);
    }

    println!("\n\x1b[34mRACA-Shell \x1b[31mv0.1.0");
    println!("\n\x1b[33mRemember to keep happy all the day when you open this shell! :)\n");

    let mut input_buf = String::new();

    print!("{}", get_prompt());

    loop {
        stdin().read_line(&mut input_buf);

        let input =
            String::from_utf8(escape_bytes::unescape(input_buf.as_bytes()).unwrap()).unwrap();

        let args = input.split(" ").map(|x| x.to_string()).collect::<Vec<_>>();

        let function = command_function_list.get(&args[0].as_str());

        if let Some(function) = function {
            function(args);
        } else if let None = run::try_run(args[0].clone()) {
            if input_buf.len() > 0 {
                println!("rash: command not found: \x1b[31m{}\x1b[0m", args[0]);
            }
        }

        print!("\x1b[0m{}", get_prompt());
    }
    //loop {}
    //raca_std::task::exit(0);
    //raca_std::print!("OK");
}
