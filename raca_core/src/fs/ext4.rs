/*
    开始吧，魔兽！！！！！！！！！！！！！
*/

use alloc::{collections::btree_map::BTreeMap, string::String, sync::Arc, vec};
use ext4_rs::{BlockDevice, Ext4};
use framework::ref_to_mut;
use spin::RwLock;

use super::{
    operation::kernel_open,
    vfs::inode::{Inode, InodeRef},
};

struct Ext4InodeIo {
    inode: InodeRef,
}

impl BlockDevice for Ext4InodeIo {
    fn read_offset(&self, offset: usize) -> alloc::vec::Vec<u8> {
        let mut buf = vec![0; 512];
        self.inode.read().read_at(offset, &mut buf);
        buf
    }

    fn write_offset(&self, offset: usize, data: &[u8]) {
        self.inode.read().write_at(offset, data);
    }
}

pub struct Ext4Volume {
    volume: Arc<Ext4>,
    virtual_inodes: BTreeMap<String, InodeRef>,
    path: String,
}

impl Ext4Volume {
    pub fn new(dev: InodeRef) -> InodeRef {
        let block_device = Arc::new(Ext4InodeIo { inode: dev });
        let volume = Arc::new(Ext4::open(block_device));
        let inode = Self {
            volume,
            virtual_inodes: BTreeMap::new(),
            path: String::new(),
        };
        let inode_ref = Arc::new(RwLock::new(inode));
        inode_ref
            .write()
            .virtual_inodes
            .insert(".".into(), inode_ref.clone());
        inode_ref
    }
}

impl Inode for Ext4Volume {
    fn when_mounted(&mut self, path: String, father: Option<InodeRef>) {
        self.path.clear();
        self.path.push_str(path.as_str());
        if let Some(father) = father {
            self.virtual_inodes.insert("..".into(), father);
        }
    }

    fn when_umounted(&mut self) {}

    fn get_path(&self) -> String {
        self.path.clone()
    }

    fn mount(&self, node: InodeRef, name: String) {
        ref_to_mut(self)
            .virtual_inodes
            .insert(name.clone(), node.clone());
    }

    fn open(&self, name: String) -> Option<InodeRef> {
        let self_inode = kernel_open(self.get_path());

        if let Some(node) = self.virtual_inodes.get(&name) {
            return Some(node.clone());
        } else {
            return self
                .volume
                .generic_open(name.as_str(), &mut 2, false, 0, &mut 0)
                .ok()
                .map(|x| {
                    let ty = self.volume.dir_has_entry(x);
                    if ty {
                        Ext4Dir::new(self.volume.clone(), x)
                    } else {
                        Ext4File::new(self.volume.clone(), x)
                    }
                });
        }
    }
}

pub struct Ext4Dir {
    volume: Arc<Ext4>,
    inode_id: u32,
    path: String,
    virtual_inodes: BTreeMap<String, InodeRef>,
}

impl Ext4Dir {
    pub fn new(volume: Arc<Ext4>, inode_id: u32) -> InodeRef {
        let i = Self {
            volume,
            inode_id,
            path: String::new(),
            virtual_inodes: BTreeMap::new(),
        };
        let inode_ref = Arc::new(RwLock::new(i));
        inode_ref
            .write()
            .virtual_inodes
            .insert(".".into(), inode_ref.clone());
        inode_ref
    }
}

impl Inode for Ext4Dir {
    fn when_mounted(&mut self, path: String, father: Option<InodeRef>) {
        self.path.clear();
        self.path.push_str(path.as_str());
        if let Some(father) = father {
            self.virtual_inodes.insert("..".into(), father);
        }
    }

    fn when_umounted(&mut self) {}

    fn get_path(&self) -> String {
        self.path.clone()
    }

    fn mount(&self, node: InodeRef, name: String) {
        ref_to_mut(self)
            .virtual_inodes
            .insert(name.clone(), node.clone());
    }

    fn open(&self, name: String) -> Option<InodeRef> {
        let self_inode = kernel_open(self.get_path());
        if let Some(node) = self.virtual_inodes.get(&name) {
            return Some(node.clone());
        } else {
            return self
                .volume
                .generic_open(name.as_str(), &mut 2, false, 0, &mut 0)
                .ok()
                .map(|x| {
                    let ty = self.volume.dir_has_entry(x);
                    if ty {
                        Ext4Dir::new(self.volume.clone(), x)
                    } else {
                        Ext4File::new(self.volume.clone(), x)
                    }
                });
        }
    }
}

pub struct Ext4File {
    volume: Arc<Ext4>,
    inode_id: u32,
    path: String,
}

impl Ext4File {
    pub fn new(volume: Arc<Ext4>, inode_id: u32) -> InodeRef {
        let i = Self {
            volume,
            inode_id,
            path: String::new(),
        };
        Arc::new(RwLock::new(i))
    }
}

impl Inode for Ext4File {
    fn when_mounted(&mut self, path: String, _father: Option<InodeRef>) {
        self.path.clear();
        self.path.push_str(path.as_str());
    }

    fn when_umounted(&mut self) {}

    fn get_path(&self) -> String {
        self.path.clone()
    }
}
