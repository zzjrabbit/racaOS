use core::sync::atomic::Ordering;

use alloc::{string::String, sync::Arc};
use fatfs::{
    DefaultTimeProvider, Dir, DirEntry, File as FileInner,
    LossyOemCpConverter, Read, Seek, SeekFrom, Write,
};
use ostd::sync::RwLock;

use crate::{FileType, InodeOperation, underlying::fat::fs::{FatDisk, FatFs}};

pub(super) struct FatDir {
    dir: RwLock<Dir<'static, FatDisk, DefaultTimeProvider, LossyOemCpConverter>>,
    cluster_size: u64,
    inode_id: u64,
    fs: Arc<FatFs>,
}

#[allow(unsafe_code)]
unsafe impl Send for FatDir {}

#[allow(unsafe_code)]
unsafe impl Sync for FatDir {}

impl FatDir {
    pub(super) fn new(
        dir: Dir<'static, FatDisk, DefaultTimeProvider, LossyOemCpConverter>,
        cluster_size: u64,
        fs: Arc<FatFs>,
    ) -> Self {
        let inode_id = fs.inode_count().fetch_add(1, Ordering::SeqCst);
        FatDir {
            dir: RwLock::new(dir),
            cluster_size,
            inode_id,
            fs,
        }
    }
}

impl InodeOperation for FatDir {
    fn create(&self, name: String, file_type: FileType) -> Option<Arc<dyn InodeOperation>> {
        let _guard = self.fs.lock();
        
        match file_type {
            FileType::File => {
                let dir = self.dir.read();
                let file = dir.create_file(&name).ok()?;
                let entry = dir
                    .iter()
                    .flatten()
                    .find(|entry| entry.file_name() == name)
                    .unwrap();
                Some(Arc::new(FatFile::new(
                    entry,
                    file,
                    self.cluster_size,
                    self.fs.clone(),
                )))
            }
            FileType::Directory => {
                let dir = self.dir.read().create_dir(&name).ok()?;
                Some(Arc::new(FatDir::new(
                    dir,
                    self.cluster_size,
                    self.fs.clone(),
                )))
            }
            _ => None,
        }
    }

    fn file_type(&self) -> FileType {
        FileType::Directory
    }

    fn lookup(&self, name: String) -> Option<Arc<dyn InodeOperation>> {
        let _guard = self.fs.lock();

        let dir = self.dir.read();
        let entry = dir
            .iter()
            .flatten()
            .find(|entry| entry.file_name() == name)?;

        let file_type = if entry.is_file() {
            FileType::File
        } else if entry.is_dir() {
            FileType::Directory
        } else {
            return None;
        };

        match file_type {
            FileType::File => {
                let file = dir.open_file(&name).ok()?;
                let file = Arc::new(FatFile::new(
                    entry,
                    file,
                    self.cluster_size,
                    self.fs.clone(),
                ));
                Some(file)
            }
            FileType::Directory => Some(Arc::new(FatDir::new(
                dir.open_dir(&name).ok()?,
                self.cluster_size,
                self.fs.clone(),
            ))),
            _ => None,
        }
    }

    fn inode_id(&self) -> u64 {
        self.inode_id
    }
    
    fn file_system(&self) -> Arc<dyn crate::FileSystem> {
        self.fs.clone()
    }
}

type FatDirEntry = DirEntry<'static, FatDisk, DefaultTimeProvider, LossyOemCpConverter>;
type FatFileInner = FileInner<'static, FatDisk, DefaultTimeProvider, LossyOemCpConverter>;

pub(super) struct FatFile {
    entry: FatDirEntry,
    file: RwLock<FatFileInner>,
    cluster_size: u64,
    inode_id: u64,
    fs: Arc<FatFs>,
}

#[allow(unsafe_code)]
unsafe impl Send for FatFile {}

#[allow(unsafe_code)]
unsafe impl Sync for FatFile {}

impl FatFile {
    pub(super) fn new(
        entry: FatDirEntry,
        file: FatFileInner,
        cluster_size: u64,
        fs: Arc<FatFs>,
    ) -> Self {
        let inode_id = fs.inode_count().fetch_add(1, Ordering::SeqCst);
        FatFile {
            entry,
            file: RwLock::new(file),
            cluster_size,
            inode_id,
            fs,
        }
    }
}

impl InodeOperation for FatFile {
    fn file_type(&self) -> FileType {
        FileType::File
    }

    fn len(&self) -> u64 {
        self.entry.len()
    }

    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> usize {
        let _guard = self.fs.lock();
                
        let mut read: u64 = 0;
        let cluster_size = self.cluster_size;
        let file_len = self.len();
        let buf_len = (buffer.len() as u64).min(file_len - offset);

        let mut file = self.file.write();

        while read < buf_len {
            let current = offset + read;
            let cluster_offset = current % cluster_size;
            let remaining = buf_len - read;

            let chunk_size = (cluster_size - cluster_offset).min(remaining);

            if file.seek(SeekFrom::Start(current)).is_err() {
                break;
            }

            if file
                .read_exact(&mut buffer[read as usize..read as usize + chunk_size as usize])
                .is_err()
            {
                break;
            }

            read += chunk_size;
        }

        read as usize
    }

    fn write_at(&self, offset: u64, buffer: &[u8]) -> usize {
        let _guard = self.fs.lock();

        let mut written: u64 = 0;
        let cluster_size = self.cluster_size;
        let file_len = self.len();
        let buf_len = (buffer.len() as u64).min(file_len - offset);

        let mut file = self.file.write();

        while written < buf_len {
            let current = offset + written;
            let cluster_offset = current % cluster_size;
            let remaining = buf_len - written;

            let chunk_size = (cluster_size - cluster_offset).min(remaining);

            if file.seek(SeekFrom::Start(current)).is_err() {
                break;
            }

            if file
                .write_all(&buffer[written as usize..written as usize + chunk_size as usize])
                .is_err()
            {
                break;
            }

            written += chunk_size;
        }

        written as usize
    }

    fn inode_id(&self) -> u64 {
        self.inode_id
    }
    
    fn file_system(&self) -> Arc<dyn crate::FileSystem> {
        self.fs.clone()
    }
}

