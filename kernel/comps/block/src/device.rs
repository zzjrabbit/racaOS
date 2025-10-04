use core::{
    fmt::Display,
    ops::Range,
    sync::atomic::{AtomicUsize, Ordering},
};

use thiserror::Error;

use crate::BlockIo;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BlockDeviceType {
    RamDisk,
    Sata,
    Nvme(usize),
}

impl Display for BlockDeviceType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        static RAM_COUNT: AtomicUsize = AtomicUsize::new(0);
        static NVME_COUNT: AtomicUsize = AtomicUsize::new(0);
        static SATA_COUNT: AtomicUsize = AtomicUsize::new(0);

        match self {
            BlockDeviceType::RamDisk => {
                write!(f, "ram{}", RAM_COUNT.fetch_add(1, Ordering::Relaxed))
            }
            BlockDeviceType::Sata => write!(
                f,
                "sd{}",
                (b'a' + SATA_COUNT.fetch_add(1, Ordering::Relaxed) as u8) as char
            ),
            BlockDeviceType::Nvme(id) => write!(
                f,
                "nvme{}n{}",
                id,
                NVME_COUNT.fetch_add(1, Ordering::Relaxed)
            ),
        }
    }
}

#[derive(Error, Debug)]
pub enum BlockDeviceError {
    #[error("Block number {0} out of {1:?}.")]
    BlockNumberOutOfBounds(u64, Range<u64>),
}

pub trait BlockDevice: Sync + Send {
    fn commit_io(&self, block_offset: u64, io: BlockIo) -> Result<(), BlockDeviceError>;

    fn metadata(&self) -> BlockMetadata;
}

pub struct BlockMetadata {
    pub total_sectors: u64,
    pub device_type: BlockDeviceType,
}
