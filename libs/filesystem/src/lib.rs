#![no_std]

extern crate alloc;

use alloc::sync::Arc;
use bitflags::bitflags;
use component::{ComponentInitError, init_component};
use spin::Lazy;
use thiserror::Error;

mod block;
mod dev;
mod ext;
mod fat;
mod part;
mod probe;
mod ramfs;
mod vfs;

pub use vfs::*;
pub use dev::init_terminal;

use crate::ramfs::RamInode;

pub type FileDescriptor = i32;

static ROOT_FS: Lazy<Arc<File>> = Lazy::new(|| {
    let inode = RamInode::new();
    File::new(Path::new("/"), inode, FileType::Directory)
});

pub fn open_file(path: &Path) -> Option<Arc<File>> {
    let parts = path.parts();
    let mut current = ROOT_FS.clone();

    for part in parts {
        current = current.lookup(&part)?;
    }

    Some(current)
}

#[init_component]
pub fn init() -> Result<(), ComponentInitError> {
    dev::init();

    ext::init();
    fat::init();

    part::init();
    probe::init();
    
    Ok(())
}

#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum FileSystemError {
    #[error("Inode not found")]
    InodeNotFound,
    #[error("Invalid arguments.")]
    InvalidArguments,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum AccessMode {
    O_RDONLY = 0,
    O_WRONLY = 1,
    O_RDWR = 2,
}

impl AccessMode {
    pub fn is_readable(&self) -> bool {
        self == &AccessMode::O_RDONLY || self == &AccessMode::O_RDWR
    }

    pub fn is_writable(&self) -> bool {
        self == &AccessMode::O_WRONLY || self == &AccessMode::O_RDWR
    }
}

impl From<AccessMode> for i32 {
    fn from(mode: AccessMode) -> Self {
        mode as i32
    }
}

impl TryFrom<i32> for AccessMode {
    type Error = ();

    fn try_from(mode: i32) -> Result<Self, Self::Error> {
        match mode & 3 {
            0 => Ok(AccessMode::O_RDONLY),
            1 => Ok(AccessMode::O_WRONLY),
            2 => Ok(AccessMode::O_RDWR),
            _ => Err(()),
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct OpenFlags: i32 {
        const O_CREAT = 64;
        const O_EXCL = 128;
        const O_TRUNC = 512;
        const O_APPEND = 1024;
        const O_NONBLOCK = 2048;
        const O_NOFOLLOW = 0x20000;
        const O_CLOEXEC = 0x80000;
        const O_DIRECTORY = 0x10000;
        const O_SYNC = 1052672;
        const O_NOCTTY = 256;
        const O_DSYNC = 4096;
        const O_ASYNC = 0x2000;
        const O_DIRECT = 0x4000;
        const O_NOATIME = 0o1000000;
        const O_PATH = 0o10000000;
    }
}

impl From<i32> for OpenFlags {
    fn from(flags: i32) -> Self {
        let bits = flags & !3;
        OpenFlags::from_bits_truncate(bits)
    }
}

bitflags! {
    pub struct InodeMode: u32 {
        const S_ISUID = 0o4000;
        const S_ISGID = 0o2000;
        const S_ISVTX = 0o1000;
        const S_IXUSR = 0o0100;
        const S_IWUSR = 0o0200;
        const S_IRUSR = 0o0400;
        const S_IXGRP = 0o0010;
        const S_IWGRP = 0o0020;
        const S_IRGRP = 0o0040;
        const S_IXOTH = 0o0001;
        const S_IWOTH = 0o0002;
        const S_IROTH = 0o0004;
    }
}

#[allow(dead_code)]
impl InodeMode {
    pub fn is_readable(&self) -> bool {
        self.contains(Self::S_IRUSR)
    }

    pub fn is_writable(&self) -> bool {
        self.contains(Self::S_IWUSR)
    }

    pub fn is_executable(&self) -> bool {
        self.contains(Self::S_IXUSR)
    }
}

impl From<u32> for InodeMode {
    fn from(mode: u32) -> Self {
        InodeMode::from_bits_truncate(mode)
    }
}
