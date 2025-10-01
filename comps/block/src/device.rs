use core::ops::Range;

use thiserror::Error;

use crate::BlockIo;

#[derive(Error, Debug)]
pub enum BlockDeviceError {
    #[error("Block number {0} out of {1:?}.")]
    BlockNumberOutOfBounds(usize, Range<usize>),
}

pub trait BlockDevice: Sync + Send {
    fn commit_io(&self, block_offset: usize, io: BlockIo) -> Result<(), BlockDeviceError>;

    fn metadata(&self) -> BlockMetadata;
}

pub struct BlockMetadata {
    pub total_sectors: usize,
}
