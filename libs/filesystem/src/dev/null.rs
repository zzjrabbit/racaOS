use alloc::sync::Arc;

use crate::{InodeOperation, dev::fs::DevFs};

pub struct NullDevice {
    fs: Arc<DevFs>,
    inode_id: u64,
}

impl NullDevice {
    pub fn new(fs: Arc<DevFs>) -> Self {
        let inode_id = fs.next_inode_id();
        NullDevice { fs, inode_id }
    }
}

impl InodeOperation for NullDevice {
    fn read_at(&self, _offset: u64, _buf: &mut [u8]) -> usize {
        0
    }

    fn write_at(&self, _offset: u64, buf: &[u8]) -> usize {
        buf.len()
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
}
