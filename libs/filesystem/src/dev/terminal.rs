use alloc::{sync::Arc, vec::Vec};
use spin::Once;

use crate::{InodeOperation, dev::fs::DevFs};

static TERMINAL: Once<fn(Vec<u8>)> = Once::new();

pub fn init_terminal(write_fn: fn(Vec<u8>)) {
    TERMINAL.call_once(|| write_fn);
}

pub struct TerminalInode {
    fs: Arc<DevFs>,
    inode_id: u64,
}

impl TerminalInode {
    pub fn new(fs: Arc<DevFs>) -> Self {
        let inode_id = fs.next_inode_id();
        TerminalInode {
            fs,
            inode_id,
        }
    }
}

impl InodeOperation for TerminalInode {
    fn read_at(&self, _offset: u64, _buf: &mut [u8]) -> usize {
        0
    }

    fn write_at(&self, _offset: u64, buf: &[u8]) -> usize {
        TERMINAL.get().unwrap()(buf.to_vec());
        buf.len()
    }

    fn len(&self) -> u64 {
        0
    }
    
    fn inode_id(&self) -> u64 {
        self.inode_id
    }
    
    fn file_system(&self) -> Arc<dyn crate::FileSystem> {
        self.fs.clone()
    }
}
