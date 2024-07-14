use alloc::{string::ToString, sync::Arc};

use crate::fs::ROOT;

use super::{inode::mount_to, root::RootFS};

pub mod block;

pub fn init() {
    crate::drivers::ahci::init();

    let dev_fs = Arc::new(RootFS::new());
    mount_to(dev_fs.clone(), ROOT.read().clone(), "dev".to_string());

    let block_i = Arc::new(block::BlockInode::new(0));
    mount_to(block_i.clone(), dev_fs, "hd0".to_string());

    //while !crate::drivers::ahci::DISK_START.is_initialized() {}
    //framework::serial_println!("{:?}",crate::drivers::ahci::DISK_START);

    let mut buf = [0; 512];
    use crate::fs::vfs::inode::Inode;
    block_i.read_at(0, &mut buf);
    framework::serial_println!("{:?}", buf);
}
