use crate::filesystem::InodeOperation;

pub struct NullDevice;

impl InodeOperation for NullDevice {
    fn read_at(&self, _offset: u64, _buf: &mut [u8]) -> usize {
        0
    }

    fn write_at(&self, _offset: u64, buf: &[u8]) -> usize {
        buf.len()
    }

    fn len(&self) -> u64 {
        0
    }
}
