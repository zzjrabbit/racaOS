use alloc::{sync::Arc, vec::Vec};
use spin::RwLock;

use crate::InodeOperation;

pub struct RamInode {
    data: RwLock<Vec<u8>>,
}

impl RamInode {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(Vec::new()),
        }
    }
}

impl InodeOperation for RamInode {
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> usize {
        let offset = offset as usize;

        let len = buf.len().min(self.data.read().len() - offset);

        for (index, byte) in buf.iter_mut().enumerate().take(len) {
            *byte = self.data.read()[offset + index];
        }

        len
    }

    fn write_at(&self, offset: u64, buf: &[u8]) -> usize {
        let offset = offset as usize;

        {
            let mut data = self.data.write();
            while data.len() < offset {
                data.push(0);
            }
        }

        let len = buf.len().min(self.data.read().len() - offset);

        for (index, byte) in buf.iter().enumerate().take(len) {
            self.data.write()[offset + index] = *byte;
        }

        if len < buf.len() {
            for byte in buf.iter().skip(len) {
                self.data.write().push(*byte);
            }
        }

        buf.len()
    }

    fn len(&self) -> u64 {
        self.data.read().len() as u64
    }

    fn create(
        &self,
        _name: alloc::string::String,
        _file_type: super::FileType,
    ) -> Option<alloc::sync::Arc<dyn InodeOperation>> {
        Some(Arc::new(Self {
            data: RwLock::new(Vec::new()),
        }))
    }

    fn remove(&self, _name: alloc::string::String) -> Option<()> {
        Some(())
    }
}
