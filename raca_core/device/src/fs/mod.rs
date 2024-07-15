use alloc::{string::ToString, sync::Arc};
use spin::{Lazy, RwLock};
use vfs::{inode::InodeRef, root::RootFS};

//mod ext2;
//mod fat32;
pub mod vfs;

pub static ROOT: Lazy<InodeRef> = Lazy::new(|| Arc::new(RwLock::new(RootFS::new())));

pub fn init() {
    crate::drivers::ahci::init();
    ROOT.read().when_mounted("/".to_string(), None);

    vfs::dev::init();
}
