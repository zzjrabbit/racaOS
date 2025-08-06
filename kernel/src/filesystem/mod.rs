use alloc::sync::Arc;
use spin::Lazy;
use thiserror::Error;

mod dev;
mod ramfs;
mod vfs;

pub use vfs::*;

use crate::filesystem::ramfs::RamInode;

pub type FileDescriptor = u32;

static ROOT_FS: Lazy<Arc<File>> = Lazy::new(|| {
    let inode = RamInode::new();
    File::new(Path::new("/"), inode, FileType::Directory)
});

pub fn open_file(path: &Path) -> Option<Arc<File>> {
    let parts = path.parts();
    let mut current = ROOT_FS.clone();

    for part in parts {
        current = current.get_child(&part)?;
    }

    Some(current)
}

pub fn init() {
    dev::init();
}

#[derive(Debug, Error)]
pub enum FileSystemError {
    #[error("Inode not found")]
    InodeNotFound,
}
