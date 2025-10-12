use crate::InodeOperation;

pub struct ZeroDevice;

impl InodeOperation for ZeroDevice {
    fn read_at(&self, _offset: u64, buf: &mut [u8]) -> usize {
        buf.fill(0);
        buf.len()
    }

    fn write_at(&self, _offset: u64, buf: &[u8]) -> usize {
        buf.len()
    }

    fn len(&self) -> u64 {
        0
    }
}
