/*
 * @file    :   pci.rs
 * @time    :   2023/08/28 13:53:40
 * @author  :   zzjcarrot
 */

// 来自rCore的PCI驱动代码

use alloc::vec::Vec;
use pci::*;
use spin::Mutex;

pub const PCI_COMMAND: u16 = 0x04;
pub const BAR0: u16 = 0x10;
pub const PCI_CAP_PTR: u16 = 0x34;
pub const PCI_INTERRUPT_LINE: u16 = 0x3c;
pub const PCI_INTERRUPT_PIN: u16 = 0x3d;
pub const PCI_COMMAND_INTX_DISABLE: u16 = 0x400;

pub const PCI_MSI_CTRL_CAP: u16 = 0x00;
pub const PCI_MSI_ADDR: u16 = 0x04;
pub const PCI_MSI_UPPER_ADDR: u16 = 0x08;
pub const PCI_MSI_DATA_32: u16 = 0x08;
pub const PCI_MSI_DATA_64: u16 = 0x0C;

pub const PCI_CAP_ID_MSI: u8 = 0x05;

pub const PCI_ACCESS: CSpaceAccessMethod = CSpaceAccessMethod::IO;

struct PortOpsImpl;

use x86_64::instructions::port::Port;

impl PortOps for PortOpsImpl {
    unsafe fn read8(&self, port: u16) -> u8 {
        Port::new(port).read()
    }
    unsafe fn read16(&self, port: u16) -> u16 {
        Port::new(port).read()
    }
    unsafe fn read32(&self, port: u16) -> u32 {
        Port::new(port).read()
    }
    unsafe fn write8(&self, port: u16, val: u8) {
        Port::new(port).write(val);
    }
    unsafe fn write16(&self, port: u16, val: u16) {
        Port::new(port).write(val);
    }
    unsafe fn write32(&self, port: u16, val: u32) {
        Port::new(port).write(val);
    }
}

/// Enable the pci device and its interrupt
/// Return assigned MSI interrupt number when applicable
unsafe fn enable(loc: Location) -> Option<usize> {
    let ops = &PortOpsImpl;
    let am = CSpaceAccessMethod::IO;

    // 23 and lower are used
    static mut MSI_IRQ: u32 = 23;

    let orig = am.read16(ops, loc, PCI_COMMAND);
    // IO Space | MEM Space | Bus Mastering | Special Cycles | PCI Interrupt Disable
    am.write32(ops, loc, PCI_COMMAND, (orig | 0x40f) as u32);

    // find MSI cap
    let mut msi_found = false;
    let mut cap_ptr = am.read8(ops, loc, PCI_CAP_PTR) as u16;
    let mut assigned_irq = None;
    while cap_ptr > 0 {
        let cap_id = am.read8(ops, loc, cap_ptr);
        if cap_id == PCI_CAP_ID_MSI {
            let orig_ctrl = am.read32(ops, loc, cap_ptr + PCI_MSI_CTRL_CAP);
            // The manual Volume 3 Chapter 10.11 Message Signalled Interrupts
            // 0 is (usually) the apic id of the bsp.
            am.write32(ops, loc, cap_ptr + PCI_MSI_ADDR, 0xfee00000 | (0 << 12));
            MSI_IRQ += 1;
            let irq = MSI_IRQ;
            assigned_irq = Some(irq as usize);
            // we offset all our irq numbers by 32
            if (orig_ctrl >> 16) & (1 << 7) != 0 {
                // 64bit
                am.write32(ops, loc, cap_ptr + PCI_MSI_DATA_64, irq + 32);
            } else {
                // 32bit
                am.write32(ops, loc, cap_ptr + PCI_MSI_DATA_32, irq + 32);
            }

            // enable MSI interrupt, assuming 64bit for now
            am.write32(ops, loc, cap_ptr + PCI_MSI_CTRL_CAP, orig_ctrl | 0x10000);
            msi_found = true;
        }
        cap_ptr = am.read8(ops, loc, cap_ptr + 1) as u16;
    }

    if !msi_found {
        // Use PCI legacy interrupt instead
        // IO Space | MEM Space | Bus Mastering | Special Cycles
        am.write32(ops, loc, PCI_COMMAND, (orig | 0xf) as u32);
    }

    assigned_irq
}

pub fn enable_device(device: &PCIDevice) -> Option<usize> {
    unsafe { enable(device.loc) }
}

static PCI_DEVICES: Mutex<Vec<PCIDevice>> = Mutex::new(Vec::new());
pub fn init() {
    let mut pci_devices = PCI_DEVICES.lock();
    let pci_iter = unsafe { scan_bus(&PortOpsImpl, CSpaceAccessMethod::IO) };
    for dev in pci_iter {
        pci_devices.push(dev.clone());
    }
    drop(pci_devices);
}

pub fn find_device_with_class(class: u8, sub_class: u8) -> Vec<PCIDevice> {
    let mut devices = Vec::new();
    let pci_devices = PCI_DEVICES.lock();
    for dev in pci_devices.iter() {
        if dev.id.class == class && dev.id.subclass == sub_class {
            devices.push(dev.clone())
        }
    }
    devices
}

pub fn find_device_with_vendor_product(vendor: u16, product: u16) -> Vec<PCIDevice> {
    let mut devices = Vec::new();
    let pci_devices = PCI_DEVICES.lock();
    for dev in pci_devices.iter() {
        if dev.id.vendor_id == vendor && dev.id.device_id == product {
            devices.push(dev.clone())
        }
    }
    devices
}

pub fn get_bar0_mem(loc: Location) -> Option<(usize, usize)> {
    unsafe { probe_function(&PortOpsImpl, loc, CSpaceAccessMethod::IO) }
        .and_then(|dev| dev.bars[0])
        .map(|bar| match bar {
            BAR::Memory(addr, len, _, _) => (addr as usize, len as usize),
            _ => unimplemented!(),
        })
}
