use crate::{BLOCK_SIZE, BlockDevice, BlockIo, BlockOperation, SECTOR_SIZE};
use alloc::sync::Arc;

use errors::{Errno, Result};
use filesystem::{DefaultFs, FileType, InodeMode, InodeOperation, Metadata};

pub(super) struct BlockInode {
    device: Arc<dyn BlockDevice>,
    inode_id: u64,
}

impl BlockInode {
    pub fn new(device: Arc<dyn BlockDevice>) -> Self {
        BlockInode {
            device,
            inode_id: DefaultFs::new().next_inode_id(),
        }
    }
}

impl InodeOperation for BlockInode {
    fn file_type(&self) -> FileType {
        FileType::BlockDevice
    }

    fn len(&self) -> u64 {
        self.device.metadata().total_sectors * SECTOR_SIZE as u64
    }

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        let len = buf.len();
        let first_block_remaining = (offset % BLOCK_SIZE as u64) as usize;

        let bio = BlockIo::new(
            BlockOperation::Read,
            (len + first_block_remaining).div_ceil(BLOCK_SIZE),
        );

        let Ok(waiter) = bio.commit(offset / BLOCK_SIZE as u64, &self.device) else {
            return Err(Errno::EIO.no_message());
        };
        waiter.wait();

        let offset = offset % BLOCK_SIZE as u64;
        bio.read(offset as usize, buf);

        Ok(len)
    }

    fn write_at(&self, offset: u64, buf: &[u8]) -> Result<usize> {
        let len = buf.len();
        let first_block_remaining = (offset % BLOCK_SIZE as u64) as usize;
        let bio = BlockIo::new(
            BlockOperation::Write,
            (len + first_block_remaining).div_ceil(BLOCK_SIZE),
        );

        let offset = offset % BLOCK_SIZE as u64;
        bio.write(offset as usize, buf);

        let Ok(waiter) = bio.commit(offset / BLOCK_SIZE as u64, &self.device) else {
            return Err(Errno::EIO.no_message());
        };
        waiter.wait();

        Ok(len)
    }

    fn file_system(&self) -> Arc<dyn filesystem::FileSystem> {
        DefaultFs::new()
    }

    fn inode_id(&self) -> u64 {
        self.inode_id
    }
    
    fn metadata(&self) -> Metadata {
        Metadata::new(InodeMode::full())
    }
}
