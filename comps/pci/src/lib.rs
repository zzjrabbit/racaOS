#![no_std]

use alloc::vec::Vec;
use component::{ComponentInitError, init_component};

#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64.rs"]
mod arch;
mod device;
mod resolver;

pub use device::*;

use resolver::PciResolver;
use spin::Lazy;

extern crate alloc;

#[init_component]
pub fn pci_init() -> Result<(), ComponentInitError> {
    arch::init();
    Lazy::force(&PCI_DEVICES);
    Ok(())
}

pub fn get_pci_devices() -> &'static [PciDevice] {
    &PCI_DEVICES
}

static PCI_DEVICES: Lazy<Vec<PciDevice>> = Lazy::new(|| {
    let devices = PciResolver::resolve();
    devices.iter().for_each(|device| log::info!("{device}"));
    devices
});
