use core::sync::atomic::{AtomicU64, Ordering};

use alloc::{string::String, sync::Arc};
use errors::Result;
use spin::Lazy;

pub trait FileSystem {
    fn name(&self) -> String;
    fn label(&self) -> String;
    fn inode_count(&self) -> u64;
    fn sync(&self) -> Result<()>;
}

pub struct DefaultFs {
    inode_count: AtomicU64,
}

impl DefaultFs {
    pub fn new() -> Arc<Self> {
        static DEFAULT_FS: Lazy<Arc<DefaultFs>> = Lazy::new(|| Arc::new(DefaultFs::new_one()));
        DEFAULT_FS.clone()
    }

    fn new_one() -> Self {
        DefaultFs {
            inode_count: AtomicU64::new(0),
        }
    }

    pub fn next_inode_id(&self) -> u64 {
        self.inode_count.fetch_add(1, Ordering::SeqCst)
    }
}

impl FileSystem for DefaultFs {
    fn inode_count(&self) -> u64 {
        self.inode_count.load(Ordering::SeqCst)
    }

    fn label(&self) -> String {
        "".into()
    }

    fn name(&self) -> String {
        "default".into()
    }

    fn sync(&self) -> Result<()> {
        Ok(())
    }
}
