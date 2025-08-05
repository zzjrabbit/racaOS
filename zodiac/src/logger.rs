use log::{LevelFilter, Metadata, Record};
use spin::Once;

static LOGGER: Logger = Logger::new();

struct Logger {
    backend: Once<&'static dyn log::Log>,
}

impl Logger {
    const fn new() -> Self {
        Self {
            backend: Once::new(),
        }
    }
}

/// Set the logger.
/// This function shouldn't be called more than once.
/// If this function isn't called, the default logger will be used.
pub fn set_logger(new_logger: &'static dyn log::Log) {
    LOGGER.backend.call_once(|| new_logger);
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        match self.backend.get() {
            Some(backend) => backend.enabled(metadata),
            None => true,
        }
    }

    fn log(&self, record: &Record) {
        match self.backend.get() {
            Some(backend) => backend.log(record),
            None => {
                let level = record.level();
                crate::console::_print(format_args!("[ {} ] {}\n", level, record.args()));
            }
        }
    }

    fn flush(&self) {
        if let Some(logger) = self.backend.get() {
            logger.flush();
        };
    }
}

pub(crate) fn init() {
    log::set_max_level(LevelFilter::Trace);
    log::set_logger(&LOGGER).unwrap();
}
