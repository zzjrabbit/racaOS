use alloc::{string::String, sync::Arc};
use ostd::{mm::Vaddr, Error as OstdError};

use crate::filesystem::FileType;

pub struct InodeData {
    inner: Arc<dyn InodeOperation>,
}

#[allow(dead_code)]
pub trait InodeOperation: Sync + Send + 'static {
    fn file_type(&self) -> FileType {
        FileType::File
    }

    fn read_at(&self, _offset: u64, _buf: &mut [u8]) -> usize {
        log::warn!("Attempt to read unreadable inodes.");
        0
    }

    fn write_at(&self, _offset: u64, _buf: &[u8]) -> usize {
        log::warn!("Attempt to write to unwritable inodes.");
        0
    }

    fn len(&self) -> u64 {
        0
    }

    fn create(&self, _name: String, _file_type: FileType) -> Option<Arc<dyn InodeOperation>> {
        log::warn!("Attempt to create sub inode for file inodes.");
        None
    }

    fn lookup(&self, _name: String) -> Option<Arc<dyn InodeOperation>> {
        log::warn!("Attempt to lookup sub inode for file inodes.");
        None
    }

    fn remove(&self, _name: String) -> Option<()> {
        log::warn!("Attempt to remove sub inode for file inodes.");
        None
    }

    fn ioctl(&self, _cmd: u32, _arg: Vaddr) -> Result<usize, OstdError> {
        log::warn!("This inode does not support ioctl.");
        Err(OstdError::AccessDenied)
    }
}

#[allow(dead_code)]
impl InodeData {
    pub(in crate::filesystem) fn new<T>(func: T) -> Self
    where
        T: InodeOperation + Send + Sync + 'static,
    {
        Self {
            inner: Arc::new(func),
        }
    }

    pub fn file_type(&self) -> FileType {
        self.inner.file_type()
    }

    pub fn read_at(&self, offset: u64, buf: &mut [u8]) -> usize {
        self.inner.read_at(offset, buf)
    }

    pub fn write_at(&self, offset: u64, buf: &[u8]) -> usize {
        self.inner.write_at(offset, buf)
    }

    pub fn len(&self) -> u64 {
        self.inner.len()
    }

    pub fn create(&self, name: String, file_type: FileType) -> Option<Arc<Self>> {
        Some(Arc::new(Self {
            inner: self.inner.create(name, file_type)?,
        }))
    }

    pub fn lookup(&self, name: String) -> Option<Arc<Self>> {
        Some(Arc::new(Self {
            inner: self.inner.lookup(name)?,
        }))
    }

    pub fn remove(&self, name: String) -> Option<()> {
        self.inner.remove(name)
    }

    pub fn ioctl(&self, cmd: u32, arg: Vaddr) -> Result<usize, OstdError> {
        self.inner.ioctl(cmd, arg)
    }
}
