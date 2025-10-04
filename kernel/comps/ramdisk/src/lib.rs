#![no_std]
#![no_main]

extern crate alloc;

use alloc::sync::Arc;
use block::{
    BLOCK_SIZE, BlockDevice, BlockDeviceError, BlockMetadata, BlockOperation, SECTOR_SIZE,
    register_device,
};
use component::{ComponentInitError, init_component};
use ostd::{boot::boot_info, mm::VmIo};

pub struct RamDisk {
    data: &'static [u8],
}

impl RamDisk {
    pub fn new(data: &'static [u8]) -> Self {
        RamDisk { data }
    }
}

impl BlockDevice for RamDisk {
    fn commit_io(
        &self,
        block_offset: u64,
        io: block::BlockIo,
    ) -> Result<(), block::BlockDeviceError> {
        if io.operation() == BlockOperation::Write {
            io.complete();
            return Err(BlockDeviceError::WriteError);
        }

        let dma_streams = io.dma_stream();

        for (id, stream) in dma_streams.iter().enumerate() {
            let block_id = block_offset + id as u64;
            stream
                .write_bytes(
                    0,
                    &self.data
                        [block_id as usize * BLOCK_SIZE..(block_id + 1) as usize * BLOCK_SIZE],
                )
                .map_err(|_| BlockDeviceError::ReadError)?;
        }

        io.complete();

        Ok(())
    }

    fn metadata(&self) -> block::BlockMetadata {
        BlockMetadata {
            total_sectors: self.data.len().div_ceil(SECTOR_SIZE) as u64,
            device_type: block::BlockDeviceType::RamDisk,
        }
    }
}

#[init_component(kthread)]
pub fn init() -> Result<(), ComponentInitError> {
    let boot_info = boot_info();
    let init_ramdisk = boot_info.initramfs.unwrap();

    let ramdisk = RamDisk::new(init_ramdisk);
    register_device(Arc::new(ramdisk));

    Ok(())
}
