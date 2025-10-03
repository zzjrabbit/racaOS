use alloc::sync::Arc;
use lwext4_rust::Ext4BlockWrapper;
use ostd::sync::RwLock;

use crate::filesystem::{ext::Lwext4Disk, File, FileSystemError, FileType, InodeOperation, Path};

pub fn parse_ext4_fs(dev: Arc<File>) -> Result<Arc<File>, FileSystemError> {
    let disk = Lwext4Disk::new(dev);
    let ext4 =
        Ext4BlockWrapper::<Lwext4Disk>::new(disk).map_err(|_| FileSystemError::InvalidArguments)?;

    let root = Ext4Root::new(ext4);
    let file = File::new(Path::new(""), root, FileType::Directory);

    Ok(file)
}

pub struct Ext4Root {
    inner: RwLock<Ext4BlockWrapper<Lwext4Disk>>,
}

#[allow(unsafe_code)]
unsafe impl Sync for Ext4Root {}

#[allow(unsafe_code)]
unsafe impl Send for Ext4Root {}

impl Ext4Root {
    pub fn new(inner: Ext4BlockWrapper<Lwext4Disk>) -> Self {
        Ext4Root {
            inner: RwLock::new(inner),
        }
    }
}

impl InodeOperation for Ext4Root {
    /*fn create(&self, name: alloc::string::String, file_type: FileType) -> Option<Arc<dyn InodeOperation>> {
        self.inner.lwext4_mount()
    }*/
}

pub struct Ext4Dir;
