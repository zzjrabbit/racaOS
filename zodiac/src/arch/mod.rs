mod error;
pub mod mem;
pub mod serial;
mod trap;

pub(crate) fn init() {
    serial::init();
    trap::init();
}
