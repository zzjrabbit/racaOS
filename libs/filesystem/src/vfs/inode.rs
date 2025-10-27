use alloc::{string::String, sync::Arc};
use bitflags::bitflags;
use errors::{Errno, Result};
use memory::Vmar;
use ostd::{
    mm::Vaddr,
    sync::{RwArc, Waker},
};

use crate::{FileSystem, FileType, IoEvent, IoctlCmd, Metadata};

pub type InodeData = Arc<dyn InodeOperation>;

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

    fn ioctl(&self, _vmar: Arc<Vmar>, _cmd: IoctlCmd, _arg: Vaddr) -> Result<usize> {
        log::warn!("This inode does not support ioctl.");
        Err(Errno::EACCES.no_message())
    }

    fn register_waker(&self, _required: IoEvent, _event: RwArc<IoEvent>, _waker: Arc<Waker>) {}

    fn inode_id(&self) -> u64;
    fn file_system(&self) -> Arc<dyn FileSystem>;
    fn metadata(&self) -> Metadata;
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
