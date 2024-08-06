use alloc::string::String;

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
        &mut self,
        path: alloc::string::String,
        _father: Option<crate::fs::vfs::inode::InodeRef>,
    ) {
        self.path.clear();
        self.path.push_str(path.as_str());
    }

    fn when_umounted(&mut self) {}

    fn get_path(&self) -> alloc::string::String {
        self.path.clone()
    }

    fn size(&self) -> usize {
        self.size
    }

    fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let offset = self.offset + offset;
        self.drive.read().read_at(offset, buf)
    }

    fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let offset = self.offset + offset;
        self.drive.read().write_at(offset, buf)
    }

    fn flush(&self) {
        self.drive.read().flush();
    }
}
