use framework::drivers::{pci::find_device_with_class, xhci::get_xhci};
use pci::BAR;

pub mod device;

pub fn init() {
    let xhci_devices = find_device_with_class(0x0C, 0x03);
    if xhci_devices.len() > 0 {
        let xhci_device = || {
            for device in xhci_devices.iter() {
                if device.id.prog_if != 0x30 {
                    continue;
                }
                if device.bars[0].is_none() && device.bars[1].is_none() {
                    continue;
                }
                if let Some(bar) = device.bars[0] {
                    match bar {
                        BAR::IO(_, _) => continue,
                        _ => {}
                    }
                }
                if let Some(bar) = device.bars[1] {
                    match bar {
                        BAR::IO(_, _) => continue,
                        _ => {}
                    }
                }
                return Some(device.clone());
            }
            return None;
        };
        let xhci_device = xhci_device();
        if let Some(xhci_device) = xhci_device {
            let bar = if let Some(bar) = xhci_device.bars[0] {
                bar
            } else {
                xhci_device.bars[1].unwrap()
            };
            let mmio = match bar {
                BAR::Memory(addr, _, _, _) => addr,
                _ => unreachable!(),
            };
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
