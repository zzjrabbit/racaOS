use core::sync::atomic::{AtomicU64, Ordering};

use alloc::sync::Arc;

use crate::{File, FileSystem, FileType, Path, dev::{null::NullDevice, terminal::TerminalInode, zero::ZeroDevice}};

pub struct DevFs {
    inode_count: AtomicU64,
}

impl DevFs {
    pub fn new() -> Arc<Self> {
        Arc::new(DevFs {
            inode_count: AtomicU64::new(0),
        })
    }
    
    pub fn next_inode_id(&self) -> u64 {
        self.inode_count.fetch_add(1, Ordering::SeqCst)
    }
}

impl DevFs {
    pub fn new_null(self: &Arc<Self>) -> Arc<File> {
        File::new(Path::from(""), NullDevice::new(self.clone()), FileType::CharDevice)
    }
    
    pub fn new_terminal(self: &Arc<Self>) -> Arc<File> {
        File::new(Path::from(""), TerminalInode::new(self.clone()), FileType::CharDevice)
    }
    
    pub fn new_zero(self: &Arc<Self>) -> Arc<File> {
        File::new(Path::from(""), ZeroDevice::new(self.clone()), FileType::BlockDevice)
    }
}

impl FileSystem for DevFs {
    fn inode_count(&self) -> u64 {
        self.inode_count.load(Ordering::SeqCst)
    }
    
    fn label(&self) -> alloc::string::String {
        "".into()
    }
    
    fn name(&self) -> alloc::string::String {
        "dev".into()
    }
}
