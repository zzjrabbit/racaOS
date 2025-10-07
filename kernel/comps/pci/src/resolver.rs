use crate::{
    PciDevice,
    arch::{read32, write32},
};
use pci_types::{capability::PciCapability, device_type::DeviceType, CommandRegister, ConfigRegionAccess, EndpointHeader, HeaderType, PciAddress, PciHeader, PciPciBridgeHeader};

use alloc::vec::Vec;

pub(crate) struct PciAccess;

impl ConfigRegionAccess for PciAccess {
    unsafe fn read(&self, address: PciAddress, offset: u16) -> u32 {
        read32(address, u32::from(offset)).unwrap()
    }

    unsafe fn write(&self, address: PciAddress, offset: u16, value: u32) {
        write32(address, u32::from(offset), value).unwrap();
    }
}

pub struct PciResolver {
    access: PciAccess,
    devices: Vec<PciDevice>,
}

impl PciResolver {
    pub fn resolve() -> Vec<PciDevice> {
        let mut resolver = Self {
            access: PciAccess,
            devices: Vec::new(),
        };

        resolver.scan_segment(0);

        resolver.devices
    }

    fn scan_segment(&mut self, segment: u16) {
        self.scan_bus(segment, 0);

        let address = PciAddress::new(segment, 0, 0, 0);
        if PciHeader::new(address).has_multiple_functions(&self.access) {
            (1..8).for_each(|i| self.scan_bus(segment, i));
        }
    }

    fn scan_bus(&mut self, segment: u16, bus: u8) {
        (0..32).for_each(|device| {
            let address = PciAddress::new(segment, bus, device, 0);
            self.scan_function(segment, bus, device, 0);

            let header = PciHeader::new(address);
            if header.has_multiple_functions(&self.access) {
                (1..8).for_each(|function| {
                    self.scan_function(segment, bus, device, function);
                });
            }
        });
    }

    fn scan_function(&mut self, segment: u16, bus: u8, device: u8, function: u8) {
        let address = PciAddress::new(segment, bus, device, function);
        let header = PciHeader::new(address);

        let (vendor_id, device_id) = header.id(&self.access);
        let (revision, class, sub_class, interface) = header.revision_and_class(&self.access);

        if vendor_id == 0xffff {
            return;
        }

        match header.header_type(&self.access) {
            HeaderType::Endpoint => {
                let mut endpoint_header = EndpointHeader::from_header(header, &self.access)
                    .expect("Invalid endpoint header");

                let device_type = DeviceType::from((class, sub_class));

                endpoint_header.capabilities(&self.access).for_each(
                    |capability| match capability {
                        PciCapability::Msi(msi) => {
                            msi.set_enabled(true, &self.access);
                        }
                        PciCapability::MsiX(mut msix) => {
                            msix.set_enabled(true, &self.access);
                        }
                        _ => {}
                    },
                );

                endpoint_header.update_command(&self.access, |command| {
                    command
                        | CommandRegister::BUS_MASTER_ENABLE
                        | CommandRegister::IO_ENABLE
                        | CommandRegister::MEMORY_ENABLE
                });

                let device = PciDevice {
                    address,
                    vendor_id,
                    device_id,
                    interface,
                    revision,
                    device_type,
                };

                self.devices.push(device);
            }
            HeaderType::PciPciBridge => {
                let bridge_header = PciPciBridgeHeader::from_header(header, &self.access)
                    .expect("Invalid PCI-PCI bridge header");

                let start_bus = bridge_header.secondary_bus_number(&self.access);
                let end_bus = bridge_header.subordinate_bus_number(&self.access);
                (start_bus..=end_bus).for_each(|bus_id| self.scan_bus(segment, bus_id));
            }
            _ => {}
        }
    }
}
