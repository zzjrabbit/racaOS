use alloc::string::String;
use core::fmt::{self, Write};
use spin::Mutex;

use crate::fs::FileDescriptor;

impl FileDescriptor {
    pub fn read_line(&self, buf: &mut String) {
        buf.clear(); // make sure that the buf is clean

        let mut tmp_buf = [0; 1];
        self.read_exact(&mut tmp_buf);

        while tmp_buf[0] != b'\n' {
            if tmp_buf[0] == 8 {
                // backspace
                let _ = buf.pop();
            } else {
                buf.push(tmp_buf[0] as char);
            }
            self.read_exact(&mut tmp_buf);
        }
    }
}

impl FileDescriptor {
    pub(self) fn stdin_read_line(&self, buf: &mut String) {
        buf.clear(); // make sure that the buf is clean

        let mut tmp_buf = [0; 1];
        self.read_exact(&mut tmp_buf);

        while tmp_buf[0] != b'\n' {
            if tmp_buf[0] == 8 {
                // backspace
                if let Some(_) = buf.pop() {
                    write!(stdout(), "{} {}", 8 as char, 8 as char).unwrap();
                }
            } else {
                write!(stdout(), "{}", tmp_buf[0] as char).unwrap();
                buf.push(tmp_buf[0] as char);
            }
            self.read_exact(&mut tmp_buf);
        }

        write!(stdout(), "\n").unwrap();
    }
}

pub struct Stdin {
    fd: FileDescriptor,
}

impl Stdin {
    pub fn read_line(&self, buf: &mut String) {
        self.fd.stdin_read_line(buf);
    }
}

pub fn stdin() -> Stdin {
    Stdin {
        fd: FileDescriptor(0, false),
    }
}

pub fn stdout() -> FileDescriptor {
    FileDescriptor(1, false)
}

struct AppOutputStream;

impl Write for AppOutputStream {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        write!(stdout(), "{}", s)
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
        $crate::io::_print(
            format_args!($($arg)*)
        )
    )
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)))
}
