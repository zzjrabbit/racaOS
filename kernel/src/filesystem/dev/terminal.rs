use crate::{filesystem::InodeOperation, terminal::terminal_write};

pub struct TerminalInode;

impl InodeOperation for TerminalInode {
    fn read_at(&self, _offset: u64, _buf: &mut [u8]) -> usize {
        0
    }

    fn write_at(&self, _offset: u64, buf: &[u8]) -> usize {
        terminal_write(buf.to_vec());
        buf.len()
    }

    fn len(&self) -> usize {
        0
    }
}
