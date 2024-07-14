use alloc::{collections::BTreeMap, string::String};
use framework::ref_to_mut;

use super::inode::{Inode, InodeRef};

pub struct RootFS {
    nodes: BTreeMap<String, InodeRef>,
    path: String,
}

impl RootFS {
    pub const fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            path: String::new(),
        }
    }
}

impl Inode for RootFS {
    fn when_mounted(&self, path: String, _father: Option<InodeRef>) {
        ref_to_mut(self).path = path;
    }

    fn when_umounted(&self) {
        for (_, node) in self.nodes.iter() {
            node.when_umounted();
        }
    }

    fn mount(&self, node: InodeRef, name: String) {
        ref_to_mut(self).nodes.insert(name, node);
    }

    fn read_at(&self, _offset: usize, _buf: &mut [u8]) {
        unimplemented!()
    }

    fn get_path(&self) -> String {
        self.path.clone()
    }
}
