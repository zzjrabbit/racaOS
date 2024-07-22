use core::fmt::{self, Write};

use spin::Mutex;

pub fn write(str: &str) -> usize {
    const WRITE_SYSCALL_ID: u64 = 0;
    crate::syscall(WRITE_SYSCALL_ID, str.as_ptr() as usize, str.len(), 0, 0, 0)
}

pub fn show_cpu_id() -> usize {
    const SHOW_CPU_ID_SYSCALL_ID: u64 = 1;
    crate::syscall(SHOW_CPU_ID_SYSCALL_ID, 0, 0, 0, 0, 0)
}

pub fn dump_hex_buffer(buffer: &[u8]) {
    const DUMP_HEX_BUFFER_SYSCALL_ID: u64 = 5;
    crate::syscall(
        DUMP_HEX_BUFFER_SYSCALL_ID,
        buffer.as_ptr() as usize,
        buffer.len(),
        0,
        0,
        0,
    );
}

struct AppOutputStream;

impl Write for AppOutputStream {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        write(s);
        Ok(())
    }
}

static OOS: Mutex<AppOutputStream> = Mutex::new(AppOutputStream);

#[inline]
pub fn _print(args: fmt::Arguments) {
    OOS.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        $crate::debug::_print(
            format_args!($($arg)*)
        )
    )
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)))
}
