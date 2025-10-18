#![no_std]

extern crate alloc;

use alloc::sync::Arc;
use component::{ComponentInitError, init_component};
use errors::{Errno, Result};
use fatfs::{FileSystem as FatFileSystem, FsOptions};
use filesystem::{File, FileSystem, FileType, Path, register_probe};

use crate::{
    fs::{FatDisk, FatFs},
    inode::FatDir,
};

mod fs;
mod inode;

#[init_component(kthread)]
pub fn init() -> ::core::result::Result<(), ComponentInitError> {
    register_probe(parse_fat);
    Ok(())
}

// TODO: Fix memory leak
pub fn parse_fat(device: Arc<File>) -> Result<Arc<File>> {
    let disk = FatDisk::new(device);

    let fs = Arc::new(FatFs::new(
        FatFileSystem::new(disk, FsOptions::new()).map_err(|_| Errno::EINVAL.no_message())?,
    ));

    Ok(File::new(
        Path::from(fs.label()),
        FatDir::new(
            fs.clone().root().root_dir(),
            fs.root().cluster_size() as u64,
            fs,
        ),
        FileType::Directory,
    ))
}
