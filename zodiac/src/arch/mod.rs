mod error;
pub mod mem;
pub mod serial;
#[cfg(feature = "smp")]
mod smp;
mod trap;

pub(crate) fn init() {
    serial::init();
    trap::init();
}

#[cfg(feature = "smp")]
pub(crate) fn init_smp() {
    smp::init();
}
