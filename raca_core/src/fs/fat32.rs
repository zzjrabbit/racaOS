use alloc::{boxed::Box, collections::BTreeMap, string::String, sync::Arc, vec::Vec};
use fatfs::*;
use framework::{ref_to_mut, ref_to_static, unsafe_trait_impl};
use spin::RwLock;

use super::{
    operation::kernel_open,
    vfs::inode::{FileInfo, Inode, InodeRef, InodeTy},
};

type FatDir = Dir<'static, InodeRefIO, NullTimeProvider, LossyOemCpConverter>;
type FatFile = File<'static, InodeRefIO, NullTimeProvider, LossyOemCpConverter>;

struct InodeRefIO {
    inode: InodeRef,
    offset: usize,
}

impl InodeRefIO {
    pub fn new(inode: InodeRef) -> Self {
        Self { inode, offset: 0 }
    }
}

impl IoBase for InodeRefIO {
    type Error = ();
}

impl Read for InodeRefIO {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        self.inode.read().read_at(self.offset, buf);

        self.seek(SeekFrom::Current(buf.len() as i64))?;
        Ok(buf.len())
    }
}

impl Write for InodeRefIO {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        self.inode.read().write_at(self.offset, buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Seek for InodeRefIO {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        match pos {
            SeekFrom::Current(i) => self.offset = (self.offset as i64 + i) as usize,
            SeekFrom::Start(i) => self.offset = i as usize,
            SeekFrom::End(i) => {
                let size = self.inode.read().size();
                self.offset = size as usize - i as usize;
            }
        }
        Ok(self.offset as u64)
    }
}

pub struct Fat32Volume {
    vol: &'static mut FileSystem<InodeRefIO>,
    virtual_inodes: BTreeMap<String, InodeRef>,
    path: String,
}

impl Fat32Volume {
    pub fn new(dev: InodeRef) -> InodeRef {
        let io = InodeRefIO::new(dev);
        let vol = Box::leak(Box::new(FileSystem::new(io, FsOptions::new()).unwrap()));

        let inode = Self {
            vol,
            virtual_inodes: BTreeMap::new(),
            path: String::new(),
        };
        let inode_ref = Arc::new(RwLock::new(inode));
        ref_to_mut(&*inode_ref.read())
            .virtual_inodes
            .insert(".".into(), inode_ref.clone());
        inode_ref
    }
}

impl Inode for Fat32Volume {
    fn when_mounted(&mut self, path: alloc::string::String, father: Option<InodeRef>) {
        self.path.clear();
        self.path.push_str(path.as_str());
        if let Some(father) = father {
            self.virtual_inodes.insert("..".into(), father);
        }
    }

    fn when_umounted(&mut self) {}

    fn get_path(&self) -> alloc::string::String {
        self.path.clone()
    }

    fn mount(&self, node: InodeRef, name: String) {
        ref_to_mut(self)
            .virtual_inodes
            .insert(name.clone(), node.clone());
    }

    fn open(&self, name: String) -> Option<InodeRef> {
        let cluster_size = self.vol.cluster_size() as usize;
        let dir = ref_to_static(self).vol.root_dir();

        let self_inode = kernel_open(self.get_path());

        if let Some(inode) = self.virtual_inodes.get(&name) {
            return Some(inode.clone());
        } else if let Ok(dir) = dir.open_dir(name.as_str()) {
            let inode = Fat32Dir::new(Arc::new(dir), cluster_size);
            inode
                .write()
                .when_mounted(self.get_path() + name.as_str() + "/", self_inode);
            return Some(inode);
        } else if let Ok(file) = dir.open_file(name.as_str()) {
            let inode = Arc::new(RwLock::new(Fat32File::new(Arc::new(file), cluster_size)));
            inode
                .write()
                .when_mounted(self.get_path() + name.as_str(), self_inode);
            return Some(inode);
        }
        //dir.
        None
    }

    fn create(&self, name: String, ty: super::vfs::inode::InodeTy) -> Option<InodeRef> {
        match ty {
            InodeTy::Dir => {
                self.vol.root_dir().create_dir(name.as_str()).ok()?;
            }
            InodeTy::File => {
                self.vol.root_dir().create_file(name.as_str()).ok()?;
            }
        }
        self.open(name)
    }

    fn inode_type(&self) -> InodeTy {
        InodeTy::Dir
    }

    fn list(&self) -> alloc::vec::Vec<FileInfo> {
        let mut vec = Vec::new();
        for (name, inode) in self.virtual_inodes.iter() {
            vec.push(FileInfo::new(name.clone(), inode.read().inode_type()));
        }
        for entry in self.vol.root_dir().iter() {
            if let Ok(entry) = entry {
                vec.push(FileInfo::new(
                    entry.file_name().clone(),
                    if entry.is_dir() {
                        InodeTy::Dir
                    } else {
                        InodeTy::File
                    },
                ))
            }
        }
        vec
    }
}

pub struct Fat32Dir {
    dir: Arc<FatDir>,
    path: String,
    cluster_size: usize,
    virtual_inodes: BTreeMap<String, InodeRef>,
}

impl Fat32Dir {
    pub(self) fn new(dir: Arc<FatDir>, cluster_size: usize) -> InodeRef {
        let inode = Self {
            dir,
            path: String::new(),
            cluster_size,
            virtual_inodes: BTreeMap::new(),
        };
        let inode_ref = Arc::new(RwLock::new(inode));
        ref_to_mut(&*inode_ref.read())
            .virtual_inodes
            .insert(".".into(), inode_ref.clone());
        inode_ref
    }
}

impl Inode for Fat32Dir {
    fn when_mounted(&mut self, path: alloc::string::String, father: Option<InodeRef>) {
        self.path.clear();
        self.path.push_str(path.as_str());
        if let Some(father) = father {
            self.virtual_inodes.insert("..".into(), father);
        }
    }

    fn when_umounted(&mut self) {}

    fn get_path(&self) -> alloc::string::String {
        self.path.clone()
    }

    fn mount(&self, node: InodeRef, name: String) {
        ref_to_mut(self)
            .virtual_inodes
            .insert(name.clone(), node.clone());
    }

    fn open(&self, name: String) -> Option<InodeRef> {
        let self_inode = kernel_open(self.get_path());

        if let Some(inode) = self.virtual_inodes.get(&name) {
            return Some(inode.clone());
        } else if let Ok(dir) = self.dir.open_dir(name.as_str()) {
            let inode = Fat32Dir::new(Arc::new(dir), self.cluster_size);
            inode
                .write()
                .when_mounted(self.get_path() + name.as_str() + "/", self_inode);
            return Some(inode);
        } else if let Ok(file) = self.dir.open_file(name.as_str()) {
            let inode = Arc::new(RwLock::new(Fat32File::new(
                Arc::new(file),
                self.cluster_size,
            )));
            inode
                .write()
                .when_mounted(self.get_path() + name.as_str(), self_inode);
            return Some(inode);
        }
        //dir.
        None
    }

    fn create(&self, name: String, ty: super::vfs::inode::InodeTy) -> Option<InodeRef> {
        match ty {
            InodeTy::Dir => {
                self.dir.create_dir(name.as_str()).ok()?;
            }
            InodeTy::File => {
                self.dir.create_file(name.as_str()).ok()?;
            }
        }
        self.open(name)
    }

    fn inode_type(&self) -> InodeTy {
        InodeTy::Dir
    }

    fn list(&self) -> alloc::vec::Vec<FileInfo> {
        let mut vec = Vec::new();
        for (name, inode) in self.virtual_inodes.iter() {
            vec.push(FileInfo::new(name.clone(), inode.read().inode_type()));
        }
        for entry in self.dir.iter() {
            if let Ok(entry) = entry {
                if entry.file_name() != "." && entry.file_name() != ".." {
                    vec.push(FileInfo::new(
                        entry.file_name().clone(),
                        if entry.is_dir() {
                            InodeTy::Dir
                        } else {
                            InodeTy::File
                        },
                    ))
                }
            }
        }
        vec
    }
}

pub struct Fat32File {
    file: Arc<FatFile>,
    path: String,
    cluster_size: usize,
}

impl Fat32File {
    pub(self) fn new(file: Arc<FatFile>, cluster_size: usize) -> Self {
        Self {
            file,
            path: String::new(),
            cluster_size,
        }
    }
}

impl Inode for Fat32File {
    fn when_mounted(&mut self, path: alloc::string::String, _father: Option<InodeRef>) {
        self.path.clear();
        self.path.push_str(path.as_str());
    }

    fn when_umounted(&mut self) {}

    fn get_path(&self) -> alloc::string::String {
        self.path.clone()
    }

    fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let mut size = 0;

        ref_to_mut(self.file.as_ref())
            .seek(SeekFrom::Start(offset as u64))
            .unwrap();
        let read_size = buf.len().min(self.size());

        let read_cnt = read_size / self.cluster_size;

        for i in 0..read_cnt {
            let offset = self.cluster_size * i;
            size += ref_to_mut(self.file.as_ref())
                .read(&mut buf[offset..offset + self.cluster_size])
                .unwrap();
        }

        let remaining = read_size % self.cluster_size;

        if remaining > 0 && buf.len() <= self.size() {
            size += ref_to_mut(self.file.as_ref())
                .read(&mut buf[read_cnt * self.cluster_size..])
                .unwrap();
        }
        size
    }

    fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let mut size = 0;
        ref_to_mut(self.file.as_ref())
            .seek(SeekFrom::Start(offset as u64))
            .unwrap();
        let write_size = buf.len();

        let write_cnt = write_size / self.cluster_size;

        for i in 0..write_cnt {
            let offset = self.cluster_size * i;
            size += ref_to_mut(self.file.as_ref())
                .write(&buf[offset..offset + self.cluster_size])
                .unwrap();
        }

        let remaining = write_size % self.cluster_size;
        if remaining > 0 {
            size += ref_to_mut(self.file.as_ref())
                .write(&buf[write_cnt * self.cluster_size..])
                .unwrap();
        }

        ref_to_mut(self.file.as_ref()).flush().unwrap();
        size
    }

    fn size(&self) -> usize {
        self.file.size().unwrap() as usize
    }
}

unsafe_trait_impl!(Fat32Volume, Sync);
unsafe_trait_impl!(InodeRefIO, Sync);
unsafe_trait_impl!(Fat32Dir, Sync);
unsafe_trait_impl!(Fat32File, Sync);

unsafe_trait_impl!(Fat32Volume, Send);
unsafe_trait_impl!(InodeRefIO, Send);
unsafe_trait_impl!(Fat32Dir, Send);
unsafe_trait_impl!(Fat32File, Send);
