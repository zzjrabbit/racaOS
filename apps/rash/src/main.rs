#![no_std]
#![no_main]

use std::{
    print, println,
    signal::{Signal, wait_for_signal},
};

use alloc::{string::String, vec::Vec};
use parser::Command;

extern crate alloc;

mod parser;

fn test() -> ! {
    std::debug::debug("test").unwrap();
    //std::println!("thread test");
    //let mut count = 0;
    /*loop {
        if count >= 100000 {
            std::debug::debug("1").unwrap();
            count = 0;
        }
        count += 1;
    }*/
    std::task::thread_exit();
}

#[unsafe(no_mangle)]
pub fn main(_handles: Vec<u32>) {
    println!(
        "
        \x1b[34m
         ██████╗   █████╗  ███████╗ ██╗  ██╗
         ██╔══██╗ ██╔══██╗ ██╔════╝ ██║  ██║
         ██████╔╝ ███████║ ███████╗ ███████║
         ██╔══██╗ ██╔══██║ ╚════██║ ██╔══██║
         ██║  ██║ ██║  ██║ ███████║ ██║  ██║
         ╚═╝  ╚═╝ ╚═╝  ╚═╝ ╚══════╝ ╚═╝  ╚═╝
        "
    );
    println!(
        "                  \x1b[31mversion: {}",
        env!("CARGO_PKG_VERSION")
    );
    println!("\n\x1b[33mRemember to keep happy all the day when you open this shell! :)\n");

    let prompt =
        "\x1b[34m╭─ \x1b[33m/home/default \x1b[0mat \x1b[34mroot@raca \n╰─\x1b[32m> \x1b[0m";

    let thread = std::task::Thread::new("test", test).unwrap();

    thread.join().unwrap();

    println!("test thread dead");

    loop {
        print!("{}", prompt);
        let mut command = String::new();
        std::read_line(&mut command).unwrap();

        let command = parser::command_parser::command(&command).unwrap();

        match command {
            Command::Echo(msg) => println!(
                "{}",
                core::str::from_utf8(&escape_bytes::unescape(msg.as_bytes()).unwrap()).unwrap()
            ),
            Command::Spawn(path, args) => {
                println!("Not supported: {} {}", path, args.join(" "));
                //std::exit(114514);
                std::task::thread_exit();
            }
            _ => {}
        }
    }
}
