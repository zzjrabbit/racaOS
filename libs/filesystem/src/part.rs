use alloc::{boxed::Box, sync::Arc, vec::Vec};
use errors::{Errno, Result};
use ostd::sync::RwLock;

use crate::{DefaultFs, File, FileSystem, InodeOperation, Metadata, vfs::InodeMode};

mod gpt;
mod mbr;

static PARSERS: RwLock<Vec<Box<dyn PartitionParser>>> = RwLock::new(Vec::new());

pub fn init() {
    gpt::init();
}

pub fn register_parser(parser: Box<dyn PartitionParser>) {
    PARSERS.write().push(parser);
}

pub fn parse_partitions(dev: Arc<File>) -> Result<Vec<Partition>> {
    for parser in PARSERS.read().iter() {
        if let Ok(partitions) = parser.parse(dev.clone()) {
            return Ok(partitions);
        }
    }
    Err(Errno::EINVAL.no_message())
}

pub trait PartitionParser: Sync + Send {
    fn parse(&self, dev: Arc<File>) -> Result<Vec<Partition>>;
}

#[derive(Clone)]
pub struct Partition {
    inner: Arc<File>,
    start: u64,
    end: u64,
    inode_id: u64,
}

impl Partition {
    pub fn new(inner: Arc<File>, start: u64, end: u64) -> Self {
        Self {
            inner,
            start,
            end,
            inode_id: DefaultFs::new().next_inode_id(),
        }
    }
}

#[allow(dead_code)]
impl Partition {
    pub fn start(&self) -> u64 {
        self.start
    }

    pub fn end(&self) -> u64 {
        self.end
    }

    pub fn into_inner(self) -> Arc<File> {
        self.inner
    }
}

impl InodeOperation for Partition {
    fn file_type(&self) -> super::FileType {
        super::FileType::BlockDevice
    }

    fn len(&self) -> u64 {
        self.end - self.start
    }

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> Result<usize> {
        if offset >= self.len() {
            return Err(Errno::EOVERFLOW.no_message());
        }

        let buf_len = buf.len();
        let inner_offset = offset + self.start;

        let buf = &mut buf[..buf_len.min((self.end - inner_offset) as usize)];
        self.inner.read_at(inner_offset, buf)
    }

    fn write_at(&self, offset: u64, buf: &[u8]) -> Result<usize> {
        if offset >= self.len() {
            return Err(Errno::EOVERFLOW.no_message());
        }

        let buf_len = buf.len();
        let inner_offset = offset + self.start;

        let buf = &buf[..buf_len.min((self.end - inner_offset) as usize)];
        self.inner.write_at(inner_offset, buf)
    }

    fn inode_id(&self) -> u64 {
        self.inode_id
    }

    fn file_system(&self) -> Arc<dyn FileSystem> {
        DefaultFs::new()
    }

    fn metadata(&self) -> Metadata {
        Metadata::new(InodeMode::full())
    }
}
