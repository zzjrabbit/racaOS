use alloc::{boxed::Box, string::String, sync::Arc};
use fatfs::{
    DefaultTimeProvider, Dir, DirEntry, File as FileInner, FileSystem, FsOptions, IoBase,
    LossyOemCpConverter, Read, Seek, SeekFrom, Write,
};
use ostd::sync::RwLock;

use crate::filesystem::{
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

    Ok(File::new(
        Path::from(root.0.volume_label()),
        FatDir::new(root.0.root_dir(), root.0.cluster_size() as u64),
        FileType::Directory,
    ))
}

struct FatRoot(FileSystem<FatDisk>);

#[allow(unsafe_code)]
unsafe impl Send for FatRoot {}

#[allow(unsafe_code)]
unsafe impl Sync for FatRoot {}

impl FatRoot {
    fn new(fs: FileSystem<FatDisk>) -> Self {
        Self(fs)
    }
}

pub struct FatDir {
    dir: RwLock<Dir<'static, FatDisk, DefaultTimeProvider, LossyOemCpConverter>>,
    cluster_size: u64,
}

#[allow(unsafe_code)]
unsafe impl Send for FatDir {}

#[allow(unsafe_code)]
unsafe impl Sync for FatDir {}

impl FatDir {
    fn new(
        dir: Dir<'static, FatDisk, DefaultTimeProvider, LossyOemCpConverter>,
        cluster_size: u64,
    ) -> Self {
        FatDir {
            dir: RwLock::new(dir),
            cluster_size,
        }
    }
}

impl InodeOperation for FatDir {
    fn create(&self, name: String, file_type: FileType) -> Option<Arc<dyn InodeOperation>> {
        match file_type {
            FileType::File => {
                let dir = self.dir.read();
                let file = dir.create_file(&name).ok()?;
                let entry = dir
                    .iter()
                    .flatten()
                    .find(|entry| entry.file_name() == name)
                    .unwrap();
                Some(Arc::new(FatFile::new(entry, file, self.cluster_size)))
            }
            FileType::Directory => {
                let dir = self.dir.read().create_dir(&name).ok()?;
                Some(Arc::new(FatDir::new(dir, self.cluster_size)))
            }
            _ => None,
        }
    }

    fn file_type(&self) -> FileType {
        FileType::Directory
    }

    fn lookup(&self, name: String) -> Option<Arc<dyn InodeOperation>> {
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
                let file = Arc::new(FatFile::new(entry, file, self.cluster_size));
                Some(file)
            }
            FileType::Directory => Some(Arc::new(FatDir::new(
                dir.open_dir(&name).ok()?,
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
    cluster_size: u64,
}

#[allow(unsafe_code)]
unsafe impl Send for FatFile {}

#[allow(unsafe_code)]
unsafe impl Sync for FatFile {}

impl FatFile {
    fn new(entry: FatDirEntry, file: FatFileInner, cluster_size: u64) -> Self {
        FatFile {
            entry,
            file: RwLock::new(file),
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

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> usize {
        if self.file.write().seek(SeekFrom::Start(offset)).is_err() {
            0
        } else {
            let mut file = self.file.write();
            let mut read = 0;

            let (offset, buf) = if offset % self.cluster_size == 0 {
                (offset, buf)
            } else {
                let cluster_id = offset / self.cluster_size;
                let cluster_offset = offset % self.cluster_size;
                let remaining = self.cluster_size - cluster_offset;

                if file
                    .seek(SeekFrom::Start(
                        cluster_id * self.cluster_size + cluster_offset,
                    ))
                    .is_err()
                {
                    return read;
                }

                if file.read(&mut buf[0..remaining as usize]).is_err() {
                    return read;
                }

                read += remaining as usize;

                (
                    cluster_id * self.cluster_size,
                    &mut buf[remaining as usize..],
                )
            };

            let end_pos = offset + buf.len() as u64;
            let buf = if end_pos % self.cluster_size == 0 {
                buf
            } else {
                let cluster_id = end_pos / self.cluster_size;
                let cluster_pos = cluster_id * self.cluster_size;

                if file.seek(SeekFrom::Start(cluster_pos)).is_err() {
                    return read;
                }

                if file
                    .read(&mut buf[(cluster_pos - offset) as usize..])
                    .is_err()
                {
                    return read;
                }

                read += (end_pos % self.cluster_size) as usize;

                &mut buf[..(cluster_pos - offset) as usize]
            };

            let len = buf.len() as u64;
            let cluster_count = len / self.cluster_size;
            let cluster_id = offset / self.cluster_size;

            for i in 0..cluster_count {
                let buffer_pos = i * self.cluster_size;

                let cluster_id = i + cluster_id;
                let cluster_pos = cluster_id * self.cluster_size;

                if file.seek(SeekFrom::Start(cluster_pos)).is_err() {
                    return read;
                }

                if file
                    .read(&mut buf[buffer_pos as usize..(buffer_pos + self.cluster_size) as usize])
                    .is_err()
                {
                    return read;
                }

                read += self.cluster_size as usize;
            }

            read
        }
    }

    fn write_at(&self, offset: u64, buf: &[u8]) -> usize {
        if self.file.write().seek(SeekFrom::Start(offset)).is_err() {
            0
        } else {
            let mut file = self.file.write();
            let mut written = 0;

            let (offset, buf) = if offset % self.cluster_size == 0 {
                (offset, buf)
            } else {
                let cluster_id = offset / self.cluster_size;
                let cluster_offset = offset % self.cluster_size;
                let remaining = self.cluster_size - cluster_offset;

                if file
                    .seek(SeekFrom::Start(
                        cluster_id * self.cluster_size + cluster_offset,
                    ))
                    .is_err()
                {
                    return written;
                }

                if file.write(&buf[0..remaining as usize]).is_err() {
                    return written;
                }

                written += remaining as usize;

                (cluster_id * self.cluster_size, &buf[remaining as usize..])
            };

            let end_pos = offset + buf.len() as u64;
            let buf = if end_pos % self.cluster_size == 0 {
                buf
            } else {
                let cluster_id = end_pos / self.cluster_size;
                let cluster_pos = cluster_id * self.cluster_size;

                if file.seek(SeekFrom::Start(cluster_pos)).is_err() {
                    return written;
                }

                if file.write(&buf[(cluster_pos - offset) as usize..]).is_err() {
                    return written;
                }

                written += (end_pos % self.cluster_size) as usize;

                &buf[..(cluster_pos - offset) as usize]
            };

            let len = buf.len() as u64;
            let cluster_count = len / self.cluster_size;
            let cluster_id = offset / self.cluster_size;

            for i in 0..cluster_count {
                let buffer_pos = i * self.cluster_size;

                let cluster_id = i + cluster_id;
                let cluster_pos = cluster_id * self.cluster_size;

                if file.seek(SeekFrom::Start(cluster_pos)).is_err() {
                    return written;
                }

                if file
                    .write(&buf[buffer_pos as usize..(buffer_pos + self.cluster_size) as usize])
                    .is_err()
                {
                    return written;
                }

                written += self.cluster_size as usize;
            }

            written
        }
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
