use alloc::string::String;
use framework::ref_to_mut;

use crate::fs::vfs::inode::{Inode, InodeRef};

pub struct PartitionInode {
    offset: usize,
    size: usize,
    drive: InodeRef,
    path: String,
}

impl PartitionInode {
    pub fn new(offset: usize, size: usize, drive: InodeRef) -> Self {
        Self {
            offset,
            size,
            drive,
            path: String::new(),
        }
    }
}

impl Inode for PartitionInode {
    fn when_mounted(
        &self,
        path: alloc::string::String,
        _father: Option<crate::fs::vfs::inode::InodeRef>,
    ) {
        ref_to_mut(self).path = path;
    }

    fn when_umounted(&self) {}

    fn get_path(&self) -> alloc::string::String {
        self.path.clone()
    }

    fn size(&self) -> usize {
        self.size
    }

    fn read_at(&self, offset: usize, buf: &mut [u8]) {
        let offset = self.offset + offset;
        self.drive.read().read_at(offset, buf);
    }

    fn write_at(&self, offset: usize, buf: &[u8]) {
        let offset = self.offset + offset;
        self.drive.read().write_at(offset, buf);
    }

    fn flush(&self) {
        self.drive.read().flush();
    }
}
