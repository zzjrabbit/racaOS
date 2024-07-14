use alloc::{string::ToString, sync::Arc};
use spin::{Lazy, RwLock};
use vfs::{
    inode::InodeRef,
    root::RootFS,
};

//mod ext2;
//mod fat32;
pub mod vfs;

pub static ROOT: Lazy<RwLock<InodeRef>> = Lazy::new(|| RwLock::new(Arc::new(RootFS::new())));

pub fn init() {
    //crate::drivers::ahci::init();
    //framework::serial_println!("{:?}",crate::drivers::ahci::DISK_START);
    ROOT.read().when_mounted("/".to_string(), None);
    //framework::serial_println!("{:?}",crate::drivers::ahci::DISK_START);

    vfs::dev::init();
}
