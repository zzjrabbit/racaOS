use alloc::{string::String, sync::Arc};
use spin::RwLock;

pub type InodeRef = Arc<RwLock<dyn Inode>>;

pub enum InodeTy {
    Dir,
    File,
}

pub trait Inode: Sync + Send {
    fn when_mounted(&self, path: String, father: Option<InodeRef>);
    fn when_umounted(&self);

    fn get_path(&self) -> String;
    fn size(&self) -> usize {
        0
    }

    fn mount(&self, _node: InodeRef, _name: String) {
        unimplemented!()
    }

    fn read_at(&self, _offset: usize, _buf: &mut [u8]) {
        unimplemented!()
    }
    fn write_at(&self, _offset: usize, _buf: &[u8]) {
        unimplemented!()
    }
    fn flush(&self) {
        unimplemented!()
    }

    fn open(&self, _name: String) -> Option<InodeRef> {
        unimplemented!()
    }
    fn create(&self, _name: String, _ty: InodeTy) -> Option<InodeRef> {
        unimplemented!()
    }
}

pub fn mount_to(node: InodeRef, to: InodeRef, name: String) {
    to.read().mount(node.clone(), name.clone());
    node.read()
        .when_mounted(to.read().get_path() + &name + "/", Some(to.clone()));
}
