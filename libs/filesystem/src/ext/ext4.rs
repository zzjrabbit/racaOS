use alloc::{string::String, sync::Arc, vec::Vec};
use lwext4_rust::{
    bindings::{O_CREAT, O_RDONLY, O_TRUNC, O_WRONLY, SEEK_SET},
    Ext4BlockWrapper, Ext4File, InodeTypes,
};
use ostd::sync::RwLock;

use crate::{ext::Lwext4Disk, File, FileSystemError, FileType, InodeOperation, Path};

pub fn parse_ext4_fs(dev: Arc<File>) -> Result<Arc<File>, FileSystemError> {
    let disk = Lwext4Disk::new(dev);
    let ext4 =
        Ext4BlockWrapper::<Lwext4Disk>::new(disk).map_err(|_| FileSystemError::InvalidArguments)?;

    let root = Arc::new(Ext4Root::new(ext4));

    static ROOTS: RwLock<Vec<Arc<Ext4Root>>> = RwLock::new(Vec::new());

    ROOTS.write().push(root.clone());

    Ok(root.root())
}

#[allow(dead_code)]
pub struct Ext4Root {
    inner: RwLock<Ext4BlockWrapper<Lwext4Disk>>,
    file: Arc<File>,
}

#[allow(unsafe_code)]
unsafe impl Sync for Ext4Root {}

#[allow(unsafe_code)]
unsafe impl Send for Ext4Root {}

impl Ext4Root {
    pub fn new(inner: Ext4BlockWrapper<Lwext4Disk>) -> Self {
        Ext4Root {
            inner: RwLock::new(inner),
            file: File::new(
                Path::new(""),
                Ext4FileWrapper::new("/", lwext4_rust::InodeTypes::EXT4_DE_DIR),
                FileType::Directory,
            ),
        }
    }

    pub fn root(&self) -> Arc<File> {
        self.file.clone()
    }
}

pub struct Ext4FileWrapper(RwLock<Ext4File>);

#[allow(unsafe_code)]
unsafe impl Sync for Ext4FileWrapper {}

#[allow(unsafe_code)]
unsafe impl Send for Ext4FileWrapper {}

impl Ext4FileWrapper {
    fn new(path: &str, inode_type: lwext4_rust::InodeTypes) -> Self {
        Ext4FileWrapper(RwLock::new(Ext4File::new(path, inode_type)))
    }
}

impl InodeOperation for Ext4FileWrapper {
    fn file_type(&self) -> FileType {
        let file_type = self.0.read().get_type();
        match file_type {
            InodeTypes::EXT4_DE_BLKDEV => FileType::BlockDevice,
            InodeTypes::EXT4_DE_CHRDEV => FileType::CharDevice,
            InodeTypes::EXT4_DE_DIR => FileType::Directory,
            InodeTypes::EXT4_DE_REG_FILE => FileType::File,
            _ => panic!("Unsupported inode type {:?} exist!", file_type),
        }
    }

    fn create(&self, name: String, file_type: FileType) -> Option<Arc<dyn InodeOperation>> {
        let path = Path::new(self.0.read().get_path().to_str().unwrap()).join(name);

        let mut file = self.0.write();

        let ext4_file_type = match file_type {
            FileType::BlockDevice => InodeTypes::EXT4_DE_BLKDEV,
            FileType::CharDevice => InodeTypes::EXT4_DE_CHRDEV,
            FileType::Directory => InodeTypes::EXT4_DE_DIR,
            FileType::File => InodeTypes::EXT4_DE_REG_FILE,
        };

        if file.check_inode_exist(path.as_str(), ext4_file_type.clone()) {
            None
        } else if ext4_file_type == InodeTypes::EXT4_DE_DIR {
            file.dir_mk(path.as_str()).ok()?;
            Some(Arc::new(Self::new(path.as_str(), InodeTypes::EXT4_DE_DIR)))
        } else {
            file.file_open(path.as_str(), O_WRONLY | O_CREAT | O_TRUNC)
                .ok()?;
            file.file_close().ok()?;
            Some(Arc::new(Self::new(path.as_str(), ext4_file_type)))
        }
    }

    fn len(&self) -> u64 {
        self.0.write().file_size()
    }

    fn lookup(&self, name: String) -> Option<Arc<dyn InodeOperation>> {
        let path = Path::new(self.0.read().get_path().to_str().unwrap()).join(name);

        let mut file = self.0.write();

        if file.check_inode_exist(path.as_str(), InodeTypes::EXT4_DE_REG_FILE) {
            Some(Arc::new(Self::new(
                path.as_str(),
                InodeTypes::EXT4_DE_REG_FILE,
            )))
        } else if file.check_inode_exist(path.as_str(), InodeTypes::EXT4_DE_DIR) {
            Some(Arc::new(Self::new(path.as_str(), InodeTypes::EXT4_DE_DIR)))
        } else {
            None
        }
    }

    fn remove(&self, name: String) -> Option<()> {
        let path = Path::new(self.0.read().get_path().to_str().unwrap()).join(name);

        let mut file = self.0.write();

        if file.check_inode_exist(path.as_str(), InodeTypes::EXT4_DE_REG_FILE) {
            file.file_remove(path.as_str()).ok().map(|_| ())
        } else if file.check_inode_exist(path.as_str(), InodeTypes::EXT4_DE_DIR) {
            file.dir_rm(path.as_str()).ok().map(|_| ())
        } else {
            None
        }
    }

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> usize {
        let mut file = self.0.write();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        if file.file_open(path, O_RDONLY).is_err() {
            return 0;
        }

        if file.file_seek(offset as i64, SEEK_SET).is_err() {
            return 0;
        }

        let r = file.file_read(buf).ok().unwrap_or(0);

        let _ = file.file_close();
        r
    }

    fn write_at(&self, offset: u64, buf: &[u8]) -> usize {
        let mut file = self.0.write();
        let path = file.get_path();
        let path = path.to_str().unwrap();
        if file.file_open(path, O_WRONLY).is_err() {
            return 0;
        }

        if file.file_seek(offset as i64, SEEK_SET).is_err() {
            return 0;
        }

        let r = file.file_write(buf).ok().unwrap_or(0);

        let _ = file.file_close();
        r
    }
}
