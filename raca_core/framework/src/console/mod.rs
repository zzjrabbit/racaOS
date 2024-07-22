use core::fmt::{self, Write};
use spin::{Lazy, Mutex};
use x86_64::instructions::interrupts;

use crate::drivers::display::Display;
use crate::terminal::Console;

mod log;

pub static CONSOLE: Lazy<Mutex<Console<Display>>> =
    Lazy::new(|| Mutex::new(Console::new(Display::new())));

pub fn init() {
    log::init();
}

#[inline]
pub fn _print(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        CONSOLE.lock().write_fmt(args).unwrap();
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        $crate::console::_print(
            format_args!($($arg)*)
        )
    )
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)))
}
