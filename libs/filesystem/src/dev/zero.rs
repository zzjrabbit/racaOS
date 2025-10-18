use alloc::sync::Arc;
use errors::Result;

use crate::{InodeOperation, Metadata, dev::fs::DevFs, vfs::InodeMode};

pub struct ZeroDevice {
    fs: Arc<DevFs>,
    inode_id: u64,
}

impl ZeroDevice {
    pub fn new(fs: Arc<DevFs>) -> Self {
        let inode_id = fs.next_inode_id();
        ZeroDevice { fs, inode_id }
    }
}

impl InodeOperation for ZeroDevice {
    fn read_at(&self, _offset: u64, buf: &mut [u8]) -> Result<usize> {
        buf.fill(0);
        Ok(buf.len())
    }

    fn write_at(&self, _offset: u64, buf: &[u8]) -> Result<usize> {
        Ok(buf.len())
    }

    fn len(&self) -> u64 {
        0
    }

    fn inode_id(&self) -> u64 {
        self.inode_id
    }

    fn file_system(&self) -> Arc<dyn crate::FileSystem> {
        self.fs.clone()
    }

    fn metadata(&self) -> Metadata {
        Metadata::new(InodeMode::full())
    }
}
