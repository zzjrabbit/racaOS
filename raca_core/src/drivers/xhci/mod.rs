use framework::drivers::{
    pci::{get_pci_device_structure, PciDeviceStructure, PCI_DEVICE_LINKEDLIST},
    xhci::get_xhci,
};

pub mod device;

pub fn init() {
    let mut list = PCI_DEVICE_LINKEDLIST.read();
    let xhci_devices = get_pci_device_structure(&mut list, 0x0C, 0x03);
    if xhci_devices.len() > 0 {
        let xhci_device = || {
            for device in xhci_devices.iter() {
                if let Some(device) = device.as_standard_device() {
                    let header = device.common_header();
                    if header.prog_if != 0x30 {
                        continue;
                    }
                    let bars = &device.standard_device_bar;
                    if bars.get_bar(0).is_err() && bars.get_bar(1).is_err() {
                        continue;
                    }
                    if bars.get_bar(0).unwrap().memory_address_size().is_none()
                        || bars.get_bar(1).unwrap().memory_address_size().is_none()
                    {
                        continue;
                    }
                    return Some(device.clone());
                }
            }
            return None;
        };
        let xhci_device = xhci_device();
        if let Some(xhci_device) = xhci_device {
            let bar = if let Ok(bar) = xhci_device.standard_device_bar.get_bar(0) {
                bar
            } else {
                xhci_device.standard_device_bar.get_bar(0).unwrap()
            };
            let mmio = bar.memory_address_size().unwrap().0;
            log::info!("MMIO address: {:x}", mmio);
            let mut xhci = get_xhci(mmio as usize);
            let operational = &mut xhci.operational;

            operational.usbcmd.update_volatile(|usb_command_register| {
                usb_command_register.set_run_stop();
            });
            while operational.usbsts.read_volatile().hc_halted() {}

            let num_ports = xhci.capability.hcsparams1.read_volatile().number_of_ports();

            log::info!("XHCI initialized! Ports: {}", num_ports);
        }
    }
}
