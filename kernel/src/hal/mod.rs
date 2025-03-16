pub mod driver;
pub mod gdt;
pub mod int;
mod mem;

pub use mem::*;
use spin::{Lazy, Mutex};

pub static BSP: Lazy<Mutex<gdt::CpuInfo>> = Lazy::new(|| Mutex::new(gdt::CpuInfo::default()));

pub fn init() {
    BSP.lock().init();
    int::init();
    driver::acpi::init();
    driver::apic::init();
}
