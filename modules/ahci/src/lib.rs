#![no_std]
#![allow(dead_code)]

extern crate alloc;

use alloc::{sync::Arc, vec::Vec};
use block::register_device;
use component::{ComponentInitError, init_component};
use ostd::{
    mm::{FrameAllocOptions, HasPaddr, USegment},
    sync::Mutex,
};
use pci::{device_type::DeviceType, get_pci_devices};

use crate::driver::Ahci;

mod cmd;
mod driver;
mod hba;
mod identify;

#[init_component(kthread)]
pub fn init() -> Result<(), ComponentInitError> {
    for device in get_pci_devices().iter() {
        if device.device_type == DeviceType::SataController {
            let bars = device.bars();
            log::info!("AHCI bars: {:x?}", bars);
            let Some(bar) = bars[5] else {
                continue;
            };
            let (address, _) = bar.unwrap_mem();

            log::info!("AHCI MMIO address: {:x}", address);

            let devices = match Ahci::new(address) {
                Ok(devices) => devices,
                Err(_) => {
                    static AHCI_MMIO_MEM: Mutex<Vec<USegment>> = Mutex::new(Vec::new());

                    let mmio = FrameAllocOptions::new().alloc_segment(4).unwrap();
                    let mmio_address = mmio.paddr();

                    device.write_bar(5, mmio_address);

                    Ahci::new_from(mmio.into()).unwrap()
                }
            };

            for ahci_device in devices {
                log::info!("AHCI device exist");
                log::info!(
                    "AHCI Device {:x}!",
                    ahci_device.identity().block_count * 512
                );
                register_device(Arc::new(ahci_device));
            }
        }
    }
    Ok(())
}
