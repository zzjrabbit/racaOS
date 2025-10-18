use alloc::sync::Arc;
use bit_field::BitField;
use driver::{DmaList, Mmio};
use ostd::mm::{Paddr, VmIo};

use super::cmd::{CommandHeader, CommandTable};
use super::driver::Ahci;

const BLOCK_SIZE: usize = 512;
const SATA_SIG_ATAPI: u32 = 0xEB140101;
const SATA_SIG_SEMB: u32 = 0xC33C0101;
const SATA_SIG_PM: u32 = 0x96690101;

pub struct HbaMemory {
    base: Paddr,
    pub capability: Mmio<u32>,
    pub global_host_control: Mmio<u32>,
    pub interrupt_status: Mmio<u32>,
    pub port_implemented: Mmio<u32>,
    pub version: Mmio<u32>,
    pub ccc_control: Mmio<u32>,
    pub ccc_ports: Mmio<u32>,
    pub em_location: Mmio<u32>,
    pub em_control: Mmio<u32>,
    pub capabilities2: Mmio<u32>,
    pub bios_os_handoff_control: Mmio<u32>,
}

impl HbaMemory {
    pub fn from_address(address: Paddr) -> ostd::Result<Self> {
        Ok(Self {
            base: address,
            capability: Mmio::new(address)?,
            global_host_control: Mmio::new(address + 4)?,
            interrupt_status: Mmio::new(address + 2 * 4)?,
            port_implemented: Mmio::new(address + 3 * 4)?,
            version: Mmio::new(address + 4 * 4)?,
            ccc_control: Mmio::new(address + 5 * 4)?,
            ccc_ports: Mmio::new(address + 6 * 4)?,
            em_location: Mmio::new(address + 7 * 4)?,
            em_control: Mmio::new(address + 8 * 4)?,
            capabilities2: Mmio::new(address + 9 * 4)?,
            bios_os_handoff_control: Mmio::new(address + 10 * 4)?,
        })
    }
}

impl HbaMemory {
    pub fn enable_ahci(&self) {
        self.global_host_control.write(&(self.global_host_control.read() | (1 << 31)));
    }
    
    pub fn disable_interrupt(&self) {
        self.global_host_control.write(&(self.global_host_control.read() & !(1 << 1)));
    }
    
    pub fn ahci_enabled(&self) -> bool {
        self.global_host_control.read().get_bit(31)
    }

    pub fn port_active(&self, port_num: usize) -> bool {
        let active = self.port_implemented.read().get_bit(port_num);
        log::info!("Port {} is active: {}", port_num, active);
        active
    }

    pub fn support_port_count(&self) -> usize {
        self.capability.read().get_bits(0..5) as usize + 1
    }

    pub fn get_port(&self, port_num: usize) -> Result<Option<HbaPort>, ostd::Error> {
        let hba_ptr = self.base;
        let offset = 0x100 + 0x80 * port_num;
        let port_address = hba_ptr + offset;

        let port = HbaPort::from_address(port_address)?;

        Ok((port.device_connected() && port.is_sata_device()).then_some(port))
    }
}

#[repr(C)]
pub struct HbaPort {
    pub command_list_base_address: Mmio<u64>,
    pub fis_base_address: Mmio<u64>,
    pub interrupt_status: Mmio<u32>,
    pub interrupt_enable: Mmio<u32>,
    pub command: Mmio<u32>,
    pub reserved: Mmio<u32>,
    pub task_file_data: Mmio<u32>,
    pub signature: Mmio<u32>,
    pub sata_status: Mmio<u32>,
    pub sata_control: Mmio<u32>,
    pub sata_error: Mmio<u32>,
    pub sata_active: Mmio<u32>,
    pub command_issue: Mmio<u32>,
    pub sata_notification: Mmio<u32>,
    pub fis_based_switch_control: Mmio<u32>,
}

impl HbaPort {
    pub fn from_address(address: Paddr) -> ostd::Result<Self> {
        Ok(Self {
            command_list_base_address: Mmio::new(address)?,
            fis_base_address: Mmio::new(address + 0x08)?,
            interrupt_status: Mmio::new(address + 0x10)?,
            interrupt_enable: Mmio::new(address + 0x14)?,
            command: Mmio::new(address + 0x18)?,
            reserved: Mmio::new(address + 0x1C)?,
            task_file_data: Mmio::new(address + 0x20)?,
            signature: Mmio::new(address + 0x24)?,
            sata_status: Mmio::new(address + 0x28)?,
            sata_control: Mmio::new(address + 0x2C)?,
            sata_error: Mmio::new(address + 0x30)?,
            sata_active: Mmio::new(address + 0x34)?,
            command_issue: Mmio::new(address + 0x38)?,
            sata_notification: Mmio::new(address + 0x3C)?,
            fis_based_switch_control: Mmio::new(address + 0x40)?,
        })
    }

    pub fn from_inner_offset<I: VmIo + 'static>(
        offset: usize,
        inner: Arc<I>,
    ) -> ostd::Result<Self> {
        Ok(Self {
            command_list_base_address: Mmio::new_from(inner.clone(), offset),
            fis_base_address: Mmio::new_from(inner.clone(), offset + 0x08),
            interrupt_status: Mmio::new_from(inner.clone(), offset + 0x10),
            interrupt_enable: Mmio::new_from(inner.clone(), offset + 0x14),
            command: Mmio::new_from(inner.clone(), offset + 0x18),
            reserved: Mmio::new_from(inner.clone(), offset + 0x1C),
            task_file_data: Mmio::new_from(inner.clone(), offset + 0x20),
            signature: Mmio::new_from(inner.clone(), offset + 0x24),
            sata_status: Mmio::new_from(inner.clone(), offset + 0x28),
            sata_control: Mmio::new_from(inner.clone(), offset + 0x2C),
            sata_error: Mmio::new_from(inner.clone(), offset + 0x30),
            sata_active: Mmio::new_from(inner.clone(), offset + 0x34),
            command_issue: Mmio::new_from(inner.clone(), offset + 0x38),
            sata_notification: Mmio::new_from(inner.clone(), offset + 0x3C),
            fis_based_switch_control: Mmio::new_from(inner.clone(), offset + 0x40),
        })
    }
}

impl HbaPort {
    pub fn start_cmd(&self) {
        let command = &self.command;
        while command.read().get_bit(15) {}
        command.write(command.read().set_bit(4, true));
        command.write(command.read().set_bit(0, true));
    }

    pub fn stop_cmd(&self) {
        let command = &self.command;
        command.write(command.read().set_bit(0, false));
        command.write(command.read().set_bit(4, false));
        while command.read().get_bit(15) || command.read().get_bit(14) {}
    }
    
    pub fn reset(&self) {
        let sata_control = &self.sata_control;
        sata_control.write(&((sata_control.read() & !0xf) | 1));
        for _ in 0..1000000 {
            core::hint::spin_loop();
        }
        sata_control.write(&(sata_control.read() & !0xf));
    }

    pub fn is_sata_device(&self) -> bool {
        !matches!(
            self.signature.read(),
            SATA_SIG_ATAPI | SATA_SIG_SEMB | SATA_SIG_PM
        )
    }

    pub fn device_connected(&self) -> bool {
        let status = self.sata_status.read();
        status.get_bits(8..12) == 1 && status.get_bits(0..4) == 3
    }
}

impl HbaPort {
    pub unsafe fn init_ahci(self) -> Ahci {
        log::info!("Initializing HBA port.");

        self.stop_cmd();
        self.reset();

        let cmd_list = DmaList::<CommandHeader>::new(32);
        let cmd_table = DmaList::<CommandTable>::new(32);
        let recieve = DmaList::<u8>::new(4096);
        
        self.command_issue.write(&0);

        self.command_list_base_address
            .write(&(cmd_list.device_address() as u64));
        self.fis_base_address.write(&(recieve.device_address() as u64));
        
        self.start_cmd();

        Ahci {
            cmd_list,
            cmd_table,
            recieve,
            port: self,
        }
    }
}
