use core::fmt::Write;

use spin::Mutex;

struct AppOutputStream;

impl Write for AppOutputStream {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        crate::print(s).unwrap();
        Ok(())
    }
}

static OOS: Mutex<AppOutputStream> = Mutex::new(AppOutputStream);

#[inline]
#[allow(static_mut_refs)]
pub fn _print(args: core::fmt::Arguments) {
    OOS.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        $crate::print::_print(
            format_args!($($arg)*)
        )
    )
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)))
}
