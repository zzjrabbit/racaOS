use alloc::{
    boxed::Box,
    string::{String, ToString},
    sync::Arc,
};
use block::SECTOR_SIZE;
use gpt_disk_io::{gpt_disk_types::BlockSize, *};

use crate::{
    part::{register_parser, Partition, PartitionParser},
    File, FileSystemError,
};

pub fn init() {
    register_parser(Box::new(GptParser));
}

pub struct GptParser;

impl PartitionParser for GptParser {
    fn parse(
        &self,
        dev: Arc<File>,
    ) -> Result<alloc::vec::Vec<super::Partition>, crate::FileSystemError> {
        let mut disk =
            Disk::new(FileWrapper(dev.clone())).map_err(|_| FileSystemError::InvalidArguments)?;

        let mut buffer = alloc::vec![0u8; 4096];
        let header = disk
            .read_primary_gpt_header(&mut buffer)
            .map_err(|_| FileSystemError::InvalidArguments)?;

        let layout = header
            .get_partition_entry_array_layout()
            .map_err(|_| FileSystemError::InvalidArguments)?;

        let part_entries = disk
            .gpt_partition_entry_array_iter(layout, &mut buffer)
            .map_err(|_| FileSystemError::InvalidArguments)?;

        Ok(part_entries
            .flatten()
            .map(|part_entry| {
                Partition::new(
                    dev.clone(),
                    part_entry.starting_lba.to_u64() * SECTOR_SIZE as u64,
                    part_entry.ending_lba.to_u64() * SECTOR_SIZE as u64,
                )
            })
            .collect())
    }
}

struct FileWrapper(Arc<File>);

impl BlockIo for FileWrapper {
    type Error = String;

    fn block_size(&self) -> gpt_disk_types::BlockSize {
        BlockSize::new(SECTOR_SIZE as u32).unwrap()
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn num_blocks(&mut self) -> Result<u64, Self::Error> {
        Ok(self.0.len() / SECTOR_SIZE as u64)
    }

    fn read_blocks(
        &mut self,
        start_lba: gpt_disk_types::Lba,
        dst: &mut [u8],
    ) -> Result<(), Self::Error> {
        let block_id = start_lba.to_u64();

        let r = self.0.read_at(block_id * SECTOR_SIZE as u64, dst);

        if r != dst.len() {
            Err("Read failed".to_string())
        } else {
            Ok(())
        }
    }

    fn write_blocks(
        &mut self,
        start_lba: gpt_disk_types::Lba,
        src: &[u8],
    ) -> Result<(), Self::Error> {
        let block_id = start_lba.to_u64();

        let r = self.0.write_at(block_id * SECTOR_SIZE as u64, src);

        if r != src.len() {
            Err("Write failed".to_string())
        } else {
            Ok(())
        }
    }
}
