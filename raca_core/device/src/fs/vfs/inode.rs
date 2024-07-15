use alloc::{string::String, sync::Arc};
use spin::RwLock;

pub type InodeRef = Arc<RwLock<dyn Inode>>;

pub trait Inode: Sync + Send {
    fn when_mounted(&self, path: String, father: Option<InodeRef>);
    fn when_umounted(&self);

    fn get_path(&self) -> String;

    fn mount(&self, _node: InodeRef, _name: String) {
        unimplemented!()
    }

    fn read_at(&self, _offset: usize, _buf: &mut [u8]) {
        unimplemented!()
    }
    fn write_at(&self, _offset: usize, _buf: &[u8]) {
        unimplemented!()
    }
}

pub fn mount_to(node: InodeRef, to: InodeRef, name: String) {
    to.read().mount(node.clone(), name.clone());
    node.read().when_mounted(to.read().get_path() + &name + "/", Some(to.clone()));
}
