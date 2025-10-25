use alloc::sync::Arc;
use errors::Result;
use spin::Once;

use crate::{InodeOperation, Metadata, dev::fs::DevFs, vfs::InodeMode};

static TERMINAL: Once<Arc<dyn Terminal>> = Once::new();

pub fn init_terminal(terminal: Arc<dyn Terminal>) {
    TERMINAL.call_once(|| terminal);
}

pub struct TerminalInode {
    fs: Arc<DevFs>,
    inode_id: u64,
}

impl TerminalInode {
    pub fn new(fs: Arc<DevFs>) -> Self {
        let inode_id = fs.next_inode_id();
        TerminalInode { fs, inode_id }
    }
}

impl InodeOperation for TerminalInode {
    fn read_at(&self, _offset: u64, buf: &mut [u8]) -> Result<usize> {
        TERMINAL.get().unwrap().read(buf)
    }

    fn write_at(&self, _offset: u64, buf: &[u8]) -> Result<usize> {
        ostd::early_print!("{}", core::str::from_utf8(buf).unwrap());
        TERMINAL.get().unwrap().write(buf)
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

    fn metadata(&self) -> Metadata {
        Metadata::new(InodeMode::full())
    }
}

pub trait Terminal: Sync + Send {
    fn read(&self, buffer: &mut [u8]) -> Result<usize>;
    fn write(&self, buffer: &[u8]) -> Result<usize>;
}
