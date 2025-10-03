use alloc::sync::Arc;
use alloc::vec::Vec;
use bit_field::BitField;
use block::{
    BLOCK_SIZE, BlockDevice, BlockDeviceError, BlockIo, BlockMetadata, BlockOperation, SECTOR_SIZE,
};
use driver::DmaList;
use ostd::mm::HasPaddr;
use ostd::mm::{
    DmaDirection, DmaStream, FrameAllocOptions, HasDaddr, HasPaddrRange, Paddr, USegment, VmIo,
};

use super::cmd::{CommandHeader, CommandTable, FisRegH2D};
use super::hba::{HbaMemory, HbaPort};
use super::identify::{Identify, IdentifyData};

const FIS_TYPE_REG_H2D: u8 = 0x27;
const CMD_READ_DMA_EXT: u8 = 0x25;
const CMD_WRITE_DMA_EXT: u8 = 0x35;
const CMD_IDENTIFY_DEVICE: u8 = 0xEC;

pub struct Ahci {
    pub port: HbaPort,
    pub cmd_list: DmaList<CommandHeader>,
    pub cmd_table: DmaList<CommandTable>,
}

unsafe impl Send for Ahci {}
unsafe impl Sync for Ahci {}

impl BlockDevice for Ahci {
    fn commit_io(&self, block_offset: u64, io: BlockIo) -> Result<(), BlockDeviceError> {
        let cmd = match io.operation() {
            BlockOperation::Read => CMD_READ_DMA_EXT,
            BlockOperation::Write => CMD_WRITE_DMA_EXT,
        };

        self.execute_command(
            cmd,
            block_offset * (BLOCK_SIZE / SECTOR_SIZE) as u64,
            io.dma_stream().iter().collect::<Vec<_>>(),
        );

        io.complete();

        Ok(())
    }

    fn metadata(&self) -> BlockMetadata {
        let identity = self.identity();
        BlockMetadata {
            total_sectors: identity.block_count,
        }
    }
}

impl Ahci {
    pub fn new(address: Paddr) -> ostd::Result<Vec<Self>> {
        let hba_memory = HbaMemory::<USegment>::from_address(address)?;

        if !hba_memory.ahci_enabled() {
            log::warn!("AHCI Not enabled");
            return Ok(Vec::new());
        }

        log::info!("AHCI supports {} ports.", hba_memory.support_port_count());

        Ok((0..hba_memory.support_port_count())
            .filter(|&port_num| hba_memory.port_active(port_num))
            .flat_map(|port_num| hba_memory.get_port(port_num))
            .flatten()
            .map(|port| unsafe { port.init_ahci() })
            .collect())
    }

    pub fn new_from(inner: USegment) -> ostd::Result<Vec<Self>> {
        let hba_memory = HbaMemory::from_inner_offset(inner.paddr(), 0, Arc::new(inner))?;

        if !hba_memory.ahci_enabled() {
            log::warn!("AHCI Not enabled");
            return Ok(Vec::new());
        }

        log::info!("AHCI supports {} ports.", hba_memory.support_port_count());

        Ok((0..hba_memory.support_port_count())
            .filter(|&port_num| hba_memory.port_active(port_num))
            .flat_map(|port_num| hba_memory.get_port(port_num))
            .flatten()
            .map(|port| unsafe { port.init_ahci() })
            .collect())
    }

    fn free_slot(&self) -> Option<usize> {
        let slots = self.port.sata_active.read() | self.port.command_issue.read();
        (0..32).find(|&i| slots & (1 << i) == 0)
    }

    pub fn identity(&self) -> IdentifyData {
        let data = DmaStream::map(
            FrameAllocOptions::new().alloc_segment(1).unwrap().into(),
            DmaDirection::Bidirectional,
            false,
        )
        .unwrap();
        self.execute_command(CMD_IDENTIFY_DEVICE, 0, alloc::vec![&data]);
        let identify: Identify = data.read_val(0).unwrap();
        IdentifyData::from(&identify)
    }

    fn execute_command(&self, command: u8, start_sector: u64, memory: Vec<&DmaStream>) {
        let index = self.free_slot().unwrap();
        log::info!("slot: {}", index);

        let mut sector_count = 0;
        for dma_stream in memory.iter() {
            sector_count += dma_stream.paddr_range().len() / SECTOR_SIZE;
        }

        log::info!("sector count: {}", sector_count);

        self.cmd_table.with_value(index, |cmd_table| {
            log::info!("ready to set prdt entries!");

            for (id, dma_stream) in memory.iter().enumerate() {
                let address = dma_stream.daddr();
                let len = dma_stream.paddr_range().len();

                cmd_table.prdt[id].data_base_address = address as u64;
                cmd_table.prdt[id].byte_count_i = len as u32;
            }

            log::info!("prdt entries written!");

            self.cmd_list.with_value(index, |cmd_header| {
                cmd_header.prdt_length = memory.len() as u16;
            });

            log::info!("prdt set!");

            let fis = &mut unsafe { *(cmd_table.cfis.as_mut_ptr() as *mut FisRegH2D) };
            fis.fis_type = FIS_TYPE_REG_H2D;
            fis.cflags = 1 << 7;
            fis.command = command;

            fis.device = match command {
                CMD_READ_DMA_EXT | CMD_WRITE_DMA_EXT => 1 << 6,
                _ => 0,
            };

            fis.sector_count = if command == CMD_IDENTIFY_DEVICE {
                0
            } else {
                sector_count as u16
            };
            fis.set_lba(start_sector);
        });

        log::info!("Issueing command.");

        self.port.command_issue.write(&(1 << index));

        self.port.start_cmd();

        // TODO: Async
        while self.port.command_issue.read().get_bit(index) {}
    }
}
