use alloc::{format, string::ToString, sync::Arc};
use spin::RwLock;

use crate::{drivers::ahci::get_hd_num, fs::ROOT};

use super::{inode::{mount_to, InodeRef}, root::RootFS};

pub mod block;

fn provide_hard_disk(hd: usize, dev_fs: InodeRef) {
    let block_i = Arc::new(RwLock::new(block::BlockInode::new(hd)));
    mount_to(block_i.clone(), dev_fs, format!("hd{}",hd));
}

fn provide_hard_disks(dev_fs: InodeRef) {
    for hd in 0..get_hd_num() {
        provide_hard_disk(hd, dev_fs.clone());
    }
}

pub fn init() {
    crate::drivers::ahci::init();

    let dev_fs = Arc::new(RwLock::new(RootFS::new()));
    mount_to(dev_fs.clone(), ROOT.clone(), "dev".to_string());

    provide_hard_disks(dev_fs.clone());
}
