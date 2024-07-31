#![no_std]
#![no_main]

use alloc::{
    collections::btree_map::BTreeMap, format, string::{String, ToString}, vec::Vec
};
use core::fmt::Write;
use raca_std::fs::{get_cwd, FileDescriptor};

extern crate alloc;

mod commands;
mod run;

fn shell_read_line(fd: &mut FileDescriptor, buf: &mut String) {
    buf.clear(); // make sure that the buf is clean

    let mut tmp_buf = [0; 1];
    fd.read(&mut tmp_buf);

    while tmp_buf[0] != b'\n' {
        if tmp_buf[0] == 8 {
            // backspace
            if let Some(_) = buf.pop() {
                write!(fd, "{} {}", 8 as char, 8 as char).unwrap();
            }
        } else {
            write!(fd, "{}", tmp_buf[0] as char).unwrap();
            buf.push(tmp_buf[0] as char);
        }
        fd.read(&mut tmp_buf);
    }
}

fn get_prompt() -> String {
    format!("\x1b[36m[\x1b[34mroot@raca \x1b[33m{}\x1b[36m]\x1b[34m:) \x1b[0m",get_cwd())
}

type CommandFunction = fn(stdio: &mut FileDescriptor, args: Vec<String>);

#[no_mangle]
pub fn main() {
    let mut command_function_list = BTreeMap::<&str, CommandFunction>::new();

    {
        use commands::*;
        command_function_list.insert("cat", cat);
        command_function_list.insert("cd", cd);
        command_function_list.insert("echo", echo);
        command_function_list.insert("ls", ls);
        command_function_list.insert("mount", mount);
        command_function_list.insert("write", write);
    }


    let mut fd = FileDescriptor::open("/dev/terminal", raca_std::fs::OpenMode::Write).unwrap();
    writeln!(fd, "\n\x1b[34mRACA-Shell \x1b[31mv0.1.0").unwrap();
    writeln!(
        fd,
        "\n\x1b[33mRemember to keep happy all the day when you open this shell! :)\n"
    )
    .unwrap();

    let mut input_buf = String::new();

    write!(fd, "{}", get_prompt()).unwrap();

    loop {
        shell_read_line(&mut fd, &mut input_buf);
        writeln!(fd).unwrap();

        let input =
            String::from_utf8(escape_bytes::unescape(input_buf.as_bytes()).unwrap()).unwrap();

        let args = input.split(" ").map(|x| x.to_string()).collect::<Vec::<_>>();
        
        let function = command_function_list.get(&args[0].as_str());

        if let Some(function) = function {
            function(&mut fd, args);
        } else if let None = run::try_run(args[0].clone()) {
            writeln!(fd, "rash: command not found: \x1b[31m{}\x1b[0m",args[0]).unwrap();
        }


        write!(fd, "\x1b[0m{}", get_prompt()).unwrap();
    }
}
