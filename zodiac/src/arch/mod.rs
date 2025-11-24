mod error;
pub mod mem;
pub mod serial;

pub(crate) fn init() {
    serial::init();
}
