use alloc::vec::Vec;
use spin::Once;

use crate::InodeOperation;

static TERMINAL: Once<fn(Vec<u8>)> = Once::new();

pub fn init_terminal(write_fn: fn(Vec<u8>)) {
    TERMINAL.call_once(|| write_fn);
}

pub struct TerminalInode;

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
}
