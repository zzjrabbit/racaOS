use alloc::sync::Arc;
use block::{BlockDevice, BlockIo, BlockOperation, BLOCK_SIZE, SECTOR_SIZE};

use crate::filesystem::InodeOperation;

pub(super) struct BlockInode {
    device: Arc<dyn BlockDevice>,
}

impl BlockInode {
    pub fn new(device: Arc<dyn BlockDevice>) -> Self {
        BlockInode { device }
    }
}

impl InodeOperation for BlockInode {
    fn file_type(&self) -> super::FileType {
        super::FileType::BlockDevice
    }

    fn len(&self) -> u64 {
        self.device.metadata().total_sectors * SECTOR_SIZE as u64
    }

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> usize {
        let len = buf.len();
        let bio = BlockIo::new(BlockOperation::Read, len.div_ceil(BLOCK_SIZE));
        
        let Ok(waiter) = bio.commit(offset / BLOCK_SIZE as u64, self.device.clone()) else {
            return 0;
        };
        waiter.wait();
        bio.read(offset as usize, buf);

        len
    }
    
    fn write_at(&self, offset: u64, buf: &[u8]) -> usize {
        let len = buf.len();
        let bio = BlockIo::new(BlockOperation::Write, len.div_ceil(BLOCK_SIZE));
        bio.write(offset as usize, buf);
        
        let Ok(waiter) = bio.commit(offset / BLOCK_SIZE as u64, self.device.clone()) else {
            return 0;
        };
        waiter.wait();

        len
    }
}
