use alloc::sync::Arc;
use errors::{Errno, Result};
use fatfs::{
    FileSystem as FatFileSystem, FsOptions,
};

use crate::{File, FileSystem, FileType, Path, probe::register_probe, underlying::fat::{fs::{FatDisk, FatFs}, inode::FatDir}};

mod inode;
mod fs;

pub fn init() {
    register_probe(parse_fat);
}

// TODO: Fix memory leak
pub fn parse_fat(device: Arc<File>) -> Result<Arc<File>> {
    let disk = FatDisk::new(device);

    let fs = Arc::new(FatFs::new(FatFileSystem::new(disk, FsOptions::new()).map_err(|_| Errno::EINVAL.no_message())?));

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
