use alloc::{string::String, sync::Arc};

pub type InodeRef = Arc<dyn Inode>;

pub trait Inode: Sync + Send {
    fn when_mounted(&self, path: String, father: Option<InodeRef>);
    fn when_umounted(&self);

    fn get_path(&self) -> String;

    fn mount(&self, node: InodeRef, name: String);

    fn read_at(&self, offset: usize, buf: &mut [u8]);
}

pub fn mount_to(node: InodeRef, to: InodeRef, name: String) {
    to.mount(node.clone(), name.clone());
    node.when_mounted(to.get_path() + &name + "/", Some(to.clone()));
}
