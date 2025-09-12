use std::{fs::File, io::Read};

mod arch;
mod cargo;
mod cli;
mod command;
mod config;

fn main() {
    let mut config = String::new();
    File::open("zoo.toml")
        .unwrap()
        .read_to_string(&mut config)
        .unwrap();

    let config = config::Config::parse(config);

    cli::cli_main(&config);
}
