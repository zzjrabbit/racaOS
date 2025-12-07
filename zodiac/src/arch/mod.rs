#[cfg(feature = "smp")]
mod boot;
pub(crate) mod context;
mod error;
mod int;
pub mod mem;
pub mod serial;

pub(crate) fn init() {
    serial::init();
    int::init();
}

#[cfg(feature = "smp")]
pub(crate) fn init_smp() {
    boot::init();
}
