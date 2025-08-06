use alloc::{string::String, sync::Arc};

use crate::filesystem::FileType;

pub struct InodeData {
    inner: Arc<dyn InodeOperation>,
}

pub trait InodeOperation: Sync + Send + 'static {
    fn read_at(&self, _offset: u64, _buf: &mut [u8]) -> usize {
        log::warn!("Attempt to read unreadable inodes.");
        0
    }
    fn write_at(&self, _offset: u64, _buf: &[u8]) -> usize {
        log::warn!("Attempt to write to unwritable inodes.");
        0
    }

    fn len(&self) -> usize {
        0
    }

    fn create(&self, _name: String, _file_type: FileType) -> Option<Arc<dyn InodeOperation>> {
        log::warn!("Attempt to create sub inode for file inodes.");
        None
    }

    fn remove(&self, _name: String) -> Option<()> {
        log::warn!("Attempt to remove sub inode for file inodes.");
        None
    }
}

impl InodeData {
    pub(in crate::filesystem) fn new<T>(func: T) -> Self
    where
        T: InodeOperation + Send + Sync + 'static,
    {
        Self {
            inner: Arc::new(func),
        }
    }

    pub fn read_at(&self, offset: u64, buf: &mut [u8]) -> usize {
        self.inner.read_at(offset, buf)
    }

    pub fn write_at(&self, offset: u64, buf: &[u8]) -> usize {
        self.inner.write_at(offset, buf)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn create(&self, name: String, file_type: FileType) -> Option<Arc<Self>> {
        Some(Arc::new(Self {
            inner: self.inner.create(name, file_type)?,
        }))
    }

    pub fn remove(&self, name: String) -> Option<()> {
        Some(self.inner.remove(name)?)
    }
}
