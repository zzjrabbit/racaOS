use core::fmt::Write;
use log::{Level, Log, Metadata, Record};
use spin::Lazy;
use spin::Mutex;
use uart_16550::SerialPort;
use x86_64::instructions::interrupts;

static SERIAL_PORT: Lazy<Mutex<SerialPort>> = Lazy::new(|| {
    let mut serial_port = unsafe { SerialPort::new(0x3f8) };
    serial_port.init();
    Mutex::new(serial_port)
});

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    interrupts::without_interrupts(|| {
        SERIAL_PORT.lock().write_fmt(args).unwrap();
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::logging::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

pub fn init() {
    static LOGGER: Logger = Logger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Debug);
}

macro_rules! log_output {
    ($color:expr, $level:expr, $args:expr, $($extra:tt)*) => {
        println!(
            "[{}] {}{}",
            format_args!("\x1b[{}m{}\x1b[0m", $color, $level),
            $args,
            format_args!($($extra)*)
        );
    };
}

struct Logger;

impl Logger {
    fn log_message(&self, record: &Record, with_location: bool) {
        let color = match record.level() {
            Level::Error => "31",
            Level::Warn => "33",
            Level::Info => "32",
            Level::Debug => "34",
            Level::Trace => "36",
        };

        if with_location {
            let file = record.file().unwrap();
            let line = record.line().unwrap();
            log_output!(color, record.level(), record.args(), ", {}:{}", file, line);
        } else {
            log_output!(color, record.level(), record.args(), "");
        }
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let with_location = matches!(record.level(), Level::Debug | Level::Trace);
            self.log_message(record, with_location);
        }
    }

    fn flush(&self) {}
}
