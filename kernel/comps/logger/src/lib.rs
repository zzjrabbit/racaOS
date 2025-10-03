#![no_std]

use alloc::{boxed::Box, format};
use component::{ComponentInitError, init_component};
use log::{Metadata, Record};
use ostd::prelude::println;

extern crate alloc;

#[init_component]
pub fn init() -> Result<(), ComponentInitError> {
    ostd::logger::inject_logger(Box::leak(Box::new(Logger::new())));
    Ok(())
}

struct Logger;

impl Logger {
    const fn new() -> Self {
        Self
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let members = env!("WORKSPACE_MEMBERS");
        members.split(',').any(|member| member == metadata.target())
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let timestamp = ostd::timer::Jiffies::elapsed().as_duration();

        let secs = timestamp.as_secs();
        let millis = timestamp.subsec_millis();

        println!(
            "{} {:<5}: {}",
            format!("[{:>6}.{:03}]", secs, millis),
            record.level(),
            record.args()
        );
    }

    fn flush(&self) {}
}
