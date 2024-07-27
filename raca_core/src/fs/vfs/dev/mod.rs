use alloc::{format, string::ToString, sync::Arc};
use spin::{Mutex, RwLock};
use terminal::Terminal;

use crate::{drivers::ahci::get_hd_num, fs::ROOT};

use super::{
    inode::{mount_to, InodeRef},
    root::RootFS,
};

pub mod block;
pub mod gpt_parser;
pub mod partition;
pub mod terminal;

pub static ROOT_PARTITION: Mutex<Option<InodeRef>> = Mutex::new(None);

fn provide_hard_disk(hd: usize, dev_fs: InodeRef) {
    let id_to_alpha = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r",
        "s", "t", "u", "v", "w", "x", "y", "z",
    ];

    let block_i = Arc::new(RwLock::new(block::BlockInode::new(hd)));
    mount_to(
        block_i.clone(),
        dev_fs.clone(),
        format!("hd{}", id_to_alpha[hd]),
    );

    let _ = gpt_parser::parse_gpt_disk(hd, block_i.clone(), dev_fs.clone());
}

fn provide_hard_disks(dev_fs: InodeRef) {
    for hd in 0..get_hd_num() {
        provide_hard_disk(hd, dev_fs.clone());
    }
}

pub fn init() {
    crate::drivers::ahci::init();

    let dev_fs = RootFS::new();
    mount_to(dev_fs.clone(), ROOT.lock().clone(), "dev".to_string());

    let terminal = Arc::new(RwLock::new(Terminal::new()));
    mount_to(terminal.clone(), dev_fs.clone(), "terminal".to_string());

    provide_hard_disks(dev_fs.clone());
}
