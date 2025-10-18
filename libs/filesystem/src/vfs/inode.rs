use alloc::{string::String, sync::Arc};
use bitflags::bitflags;
use errors::{Errno, Result};
use ostd::mm::Vaddr;

use crate::{FileSystem, FileType, Metadata};

pub struct InodeData {
    inner: Arc<dyn InodeOperation>,
}

#[allow(dead_code)]
pub trait InodeOperation: Sync + Send + 'static {
    fn file_type(&self) -> FileType {
        FileType::File
    }

    fn read_at(&self, _offset: u64, _buf: &mut [u8]) -> Result<usize> {
        log::warn!("Attempt to read unreadable inodes.");
        Err(Errno::EACCES.no_message())
    }

    fn write_at(&self, _offset: u64, _buf: &[u8]) -> Result<usize> {
        log::warn!("Attempt to write to unwritable inodes.");
        Err(Errno::EACCES.no_message())
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

    fn ioctl(&self, _cmd: u32, _arg: Vaddr) -> Result<usize> {
        log::warn!("This inode does not support ioctl.");
        Err(Errno::EACCES.no_message())
    }

    fn inode_id(&self) -> u64;
    fn file_system(&self) -> Arc<dyn FileSystem>;
    fn metadata(&self) -> Metadata;
}

#[allow(dead_code)]
impl InodeData {
    pub(crate) fn new<T>(func: T) -> Self
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

    pub fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        self.inner.read_at(offset, buf)
    }

    pub fn write_at(&self, offset: u64, buf: &[u8]) -> Result<usize> {
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

    pub fn ioctl(&self, cmd: u32, arg: Vaddr) -> Result<usize> {
        self.inner.ioctl(cmd, arg)
    }
    
    pub fn metadata(&self) -> Metadata {
        self.inner.metadata()
    }
    
    pub fn inode_id(&self) -> u64 {
        self.inner.inode_id()
    }
}

bitflags! {
    pub struct InodeMode: u16 {
        /// set-user-ID
        const S_ISUID = 0o4000;
        /// set-group-ID
        const S_ISGID = 0o2000;
        /// sticky bit
        const S_ISVTX = 0o1000;
        /// read by owner
        const S_IRUSR = 0o0400;
        /// write by owner
        const S_IWUSR = 0o0200;
        /// execute/search by owner
        const S_IXUSR = 0o0100;
        /// read by group
        const S_IRGRP = 0o0040;
        /// write by group
        const S_IWGRP = 0o0020;
        /// execute/search by group
        const S_IXGRP = 0o0010;
        /// read by others
        const S_IROTH = 0o0004;
        /// write by others
        const S_IWOTH = 0o0002;
        /// execute/search by others
        const S_IXOTH = 0o0001;
    }
}

impl InodeMode {
    pub fn full() -> Self {
        Self::from_bits_truncate(0o777)
    }
}

impl InodeMode {
    pub fn is_owner_readable(&self) -> bool {
        self.contains(Self::S_IRUSR)
    }

    pub fn is_owner_writable(&self) -> bool {
        self.contains(Self::S_IWUSR)
    }

    pub fn is_owner_executable(&self) -> bool {
        self.contains(Self::S_IXUSR)
    }

    pub fn is_group_readable(&self) -> bool {
        self.contains(Self::S_IRGRP)
    }

    pub fn is_group_writable(&self) -> bool {
        self.contains(Self::S_IWGRP)
    }

    pub fn is_group_executable(&self) -> bool {
        self.contains(Self::S_IXGRP)
    }

    pub fn is_other_readable(&self) -> bool {
        self.contains(Self::S_IROTH)
    }

    pub fn is_other_writable(&self) -> bool {
        self.contains(Self::S_IWOTH)
    }

    pub fn is_other_executable(&self) -> bool {
        self.contains(Self::S_IXOTH)
    }

    pub fn has_sticky_bit(&self) -> bool {
        self.contains(Self::S_ISVTX)
    }

    pub fn has_set_uid(&self) -> bool {
        self.contains(Self::S_ISUID)
    }

    pub fn has_set_gid(&self) -> bool {
        self.contains(Self::S_ISGID)
    }
}
