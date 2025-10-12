use core::fmt::{self, Display};
use pci_types::{
    Bar, DeviceId, DeviceRevision, EndpointHeader, Interface, MAX_BARS, PciAddress, PciHeader,
    VendorId, device_type::DeviceType,
};

use crate::resolver::PciAccess;

#[derive(Debug)]
pub struct PciDevice {
    pub address: PciAddress,
    pub vendor_id: VendorId,
    pub device_id: DeviceId,
    pub interface: Interface,
    pub revision: DeviceRevision,
    pub device_type: DeviceType,
}

impl PciDevice {
    /// # Panics
    /// If the device is invalid, the function will panic.
    #[must_use]
    pub fn bars(&self) -> [Option<Bar>; MAX_BARS] {
        let mut bars = [None; 6];
        let mut skip_next = false;

        let header = PciHeader::new(self.address);
        let header = EndpointHeader::from_header(header, PciAccess);
        let header = header.unwrap();

        for (index, bar_slot) in bars.iter_mut().enumerate() {
            if skip_next {
                skip_next = false;
                continue;
            }
            #[allow(clippy::cast_possible_truncation)]
            let bar = header.bar(index as u8, &PciAccess);
            if let Some(Bar::Memory64 { .. }) = bar {
                skip_next = true;
            }
            *bar_slot = bar;
        }

        bars
    }

    /// # Panics
    /// If the device is invalid, the function will panic.
    /// If the slot is invalid, the function will panic.
    /// If the value is invalid, the function will panic.
    pub fn write_bar(&self, slot: u8, value: usize) {
        unsafe {
            let header = PciHeader::new(self.address);
            let header = EndpointHeader::from_header(header, PciAccess);
            header.unwrap().write_bar(slot, PciAccess, value).unwrap();
        }
    }
}

impl Display for PciDevice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}:{}.{}: {:?} [{:04x}:{:04x}] (rev: {:02x})",
            self.address.bus(),
            self.address.device(),
            self.address.function(),
            self.device_type,
            self.vendor_id,
            self.device_id,
            self.revision,
        )
    }
}
