use core::fmt::{self, Arguments, Write};

use spin::Mutex;

struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &c in s.as_bytes() {
            crate::hal::device::serial::send(c);
        }
        Ok(())
    }
}

static STDOUT: Mutex<Stdout> = Mutex::new(Stdout);

#[doc(hidden)]
pub fn _print(args: Arguments) {
    crate::hal::without_interrupts(|| STDOUT.lock().write_fmt(args).unwrap());
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::_print(format_args!($fmt $(, $($arg)+)?))
    }
}

#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::_print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?))
    }
}
