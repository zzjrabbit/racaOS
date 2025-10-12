use alloc::{boxed::Box, string::String, sync::Arc};
use fatfs::{
    DefaultTimeProvider, Dir, DirEntry, File as FileInner, FileSystem, FsOptions, IoBase,
    LossyOemCpConverter, Read, Seek, SeekFrom, Write,
};
use ostd::sync::{Mutex, RwLock};

use crate::{
    probe::register_probe, File, FileSystemError, FileType, InodeOperation, Path,
};

pub fn init() {
    register_probe(parse_fat);
}

// TODO: Fix memory leak
pub fn parse_fat(device: Arc<File>) -> Result<Arc<File>, FileSystemError> {
    let disk = FatDisk { device, offset: 0 };

    let root = Box::leak(Box::new(FatRoot::new(
        FileSystem::new(disk, FsOptions::new()).map_err(|error| {
            log::error!("Fat error: {:?}!", error);
            FileSystemError::InodeNotFound
        })?,
    )));
    
    let root_lock = Arc::new(Mutex::new(()));

    Ok(File::new(
        Path::from(root.0.volume_label()),
        FatDir::new(root.0.root_dir(), root_lock.clone(), root.0.cluster_size() as u64),
        FileType::Directory,
    ))
}

struct FatRoot(FileSystem<FatDisk>);

impl FatRoot {
    fn new(fs: FileSystem<FatDisk>) -> Self {
        Self(fs)
    }
}

pub struct FatDir {
    dir: RwLock<Dir<'static, FatDisk, DefaultTimeProvider, LossyOemCpConverter>>,
    root_lock: Arc<Mutex<()>>,
    cluster_size: u64,
}

#[allow(unsafe_code)]
unsafe impl Send for FatDir {}

#[allow(unsafe_code)]
unsafe impl Sync for FatDir {}

impl FatDir {
    fn new(
        dir: Dir<'static, FatDisk, DefaultTimeProvider, LossyOemCpConverter>,
        root_lock: Arc<Mutex<()>>,
        cluster_size: u64,
    ) -> Self {
        FatDir {
            dir: RwLock::new(dir),
            root_lock,
            cluster_size,
        }
    }
}

impl InodeOperation for FatDir {
    fn create(&self, name: String, file_type: FileType) -> Option<Arc<dyn InodeOperation>> {
        let _guard = self.root_lock.lock();
        
        match file_type {
            FileType::File => {
                let dir = self.dir.read();
                let file = dir.create_file(&name).ok()?;
                let entry = dir
                    .iter()
                    .flatten()
                    .find(|entry| entry.file_name() == name)
                    .unwrap();
                Some(Arc::new(FatFile::new(entry, file, self.root_lock.clone(), self.cluster_size)))
            }
            FileType::Directory => {
                let dir = self.dir.read().create_dir(&name).ok()?;
                Some(Arc::new(FatDir::new(dir, self.root_lock.clone(), self.cluster_size)))
            }
            _ => None,
        }
    }

    fn file_type(&self) -> FileType {
        FileType::Directory
    }

    fn lookup(&self, name: String) -> Option<Arc<dyn InodeOperation>> {
        let _guard = self.root_lock.lock();
        
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
                let file = Arc::new(FatFile::new(entry, file, self.root_lock.clone(), self.cluster_size));
                Some(file)
            }
            FileType::Directory => Some(Arc::new(FatDir::new(
                dir.open_dir(&name).ok()?,
                self.root_lock.clone(),
                self.cluster_size,
            ))),
            _ => None,
        }
    }
}

type FatDirEntry = DirEntry<'static, FatDisk, DefaultTimeProvider, LossyOemCpConverter>;
type FatFileInner = FileInner<'static, FatDisk, DefaultTimeProvider, LossyOemCpConverter>;

pub struct FatFile {
    entry: FatDirEntry,
    file: RwLock<FatFileInner>,
    root_lock: Arc<Mutex<()>>,
    cluster_size: u64,
}

#[allow(unsafe_code)]
unsafe impl Send for FatFile {}

#[allow(unsafe_code)]
unsafe impl Sync for FatFile {}

impl FatFile {
    fn new(entry: FatDirEntry, file: FatFileInner, root_lock: Arc<Mutex<()>>, cluster_size: u64) -> Self {
        FatFile {
            entry,
            file: RwLock::new(file),
            root_lock,
            cluster_size,
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
        let _guard = self.root_lock.lock();
        
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
        let _guard = self.root_lock.lock();
        
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
}

struct FatDisk {
    device: Arc<File>,
    offset: u64,
}

impl IoBase for FatDisk {
    type Error = ();
}

impl Read for FatDisk {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let r = self.device.read_at(self.offset, buf);
        self.offset += r as u64;
        Ok(r)
    }
}

impl Write for FatDisk {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let r = self.device.write_at(self.offset, buf);
        self.offset += r as u64;
        Ok(r)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Seek for FatDisk {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        let new_offset = match pos {
            SeekFrom::Current(offset) => self.offset.checked_add_signed(offset).ok_or(())?,
            SeekFrom::Start(offset) => offset,
            SeekFrom::End(offset) => self.device.len().checked_add_signed(offset).ok_or(())?,
        };
        self.offset = new_offset;
        Ok(new_offset)
    }
}
