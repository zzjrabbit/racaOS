use core::sync::atomic::{AtomicU64, Ordering};

use alloc::{boxed::Box, sync::Arc};
use errors::Errno;
use fatfs::{FileSystem as FatFileSystem, IoBase, Read, Seek, SeekFrom, Write};
use ostd::sync::{Mutex, MutexGuard};

use crate::{File, FileSystem};

pub struct FatFs {
    inode_count: AtomicU64,
    lock: Mutex<()>,
    root: &'static FatFileSystem<FatDisk>,
}

impl FatFs {
    pub fn new(root: FatFileSystem<FatDisk>) -> Self {
        FatFs {
            inode_count: AtomicU64::new(0),
            lock: Mutex::new(()),
            root: Box::leak(Box::new(root)),
        }
    }

    pub fn lock(&self) -> MutexGuard<()> {
        self.lock.lock()
    }

    pub fn inode_count(&self) -> &AtomicU64 {
        &self.inode_count
    }

    pub fn root(&self) -> &'static FatFileSystem<FatDisk> {
        self.root
    }
}

impl FileSystem for FatFs {
    fn inode_count(&self) -> u64 {
        self.inode_count().load(Ordering::SeqCst)
    }

    fn name(&self) -> alloc::string::String {
        "fat".into()
    }

    fn label(&self) -> alloc::string::String {
        self.root.volume_label()
    }

    fn sync(&self) -> errors::Result<()> {
        let root_dir = self.root.root_dir();

        fn sync_dir(
            dir: fatfs::Dir<'_, FatDisk, fatfs::NullTimeProvider, fatfs::LossyOemCpConverter>,
        ) -> errors::Result<()> {
            for entry in dir.iter().flatten() {
                let name = entry.file_name();
                if entry.is_dir() {
                    let sub_dir = dir.open_dir(&name).unwrap();
                    sync_dir(sub_dir)?;
                } else {
                    let mut file = dir.open_file(&name).unwrap();
                    file.flush().map_err(|_| Errno::EIO.no_message())?;
                }
            }

            Ok(())
        }

        sync_dir(root_dir)
    }
}

pub(super) struct FatDisk {
    device: Arc<File>,
    offset: u64,
}

impl FatDisk {
    pub(super) fn new(device: Arc<File>) -> Self {
        FatDisk { device, offset: 0 }
    }
}

impl IoBase for FatDisk {
    type Error = ();
}

impl Read for FatDisk {
    fn read(&mut self, buf: &mut [u8]) -> ::core::result::Result<usize, Self::Error> {
        let r = self.device.read_at(self.offset, buf).map_err(|_| ())?;
        self.offset += r as u64;
        Ok(r)
    }
}

impl Write for FatDisk {
    fn write(&mut self, buf: &[u8]) -> ::core::result::Result<usize, Self::Error> {
        let r = self.device.write_at(self.offset, buf).map_err(|_| ())?;
        self.offset += r as u64;
        Ok(r)
    }

    fn flush(&mut self) -> ::core::result::Result<(), Self::Error> {
        Ok(())
    }
}

impl Seek for FatDisk {
    fn seek(&mut self, pos: SeekFrom) -> ::core::result::Result<u64, Self::Error> {
        let new_offset = match pos {
            SeekFrom::Current(offset) => self.offset.checked_add_signed(offset).ok_or(())?,
            SeekFrom::Start(offset) => offset,
            SeekFrom::End(offset) => self.device.len().checked_add_signed(offset).ok_or(())?,
        };
        self.offset = new_offset;
        Ok(new_offset)
    }
}
