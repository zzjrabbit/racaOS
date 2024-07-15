use alloc::{string::String, vec::Vec};
use framework::ref_to_mut;

use crate::fs::vfs::{
    cache::{Cache512B, CacheManager},
    inode::Inode,
};

pub struct BlockInode {
    pub hd: usize,
    pub cache_manager: CacheManager<Cache512B>,
    pub path: String,
}

impl BlockInode {
    pub fn new(hd: usize) -> Self {
        Self {
            hd,
            cache_manager: CacheManager::new(),
            path: String::new(),
        }
    }
}

impl Inode for BlockInode {
    fn when_mounted(
        &self,
        path: alloc::string::String,
        _father: Option<crate::fs::vfs::inode::InodeRef>,
    ) {
        ref_to_mut(self).path = path;
    }

    fn when_umounted(&self) {}

    fn get_path(&self) -> alloc::string::String {
        self.path.clone()
    }

    fn read_at(&self, offset: usize, buf: &mut [u8]) {
        let start = offset;
        let end = start + buf.len();

        let start_sector_read_start = start % 512;
        //let end_sector_read_end = end % 512;

        let start_sector_id = start / 512;
        let end_sector_id = (end - 1) / 512;

        let buffer_size = (end_sector_id - start_sector_id + 1) * 512;
        let mut tmp = Vec::new();
        for _ in 0..buffer_size {
            tmp.push(0);
        }
        let tmp = tmp.leak();

        crate::drivers::ahci::read_block(0, 0, tmp).unwrap();

        for i in 0..(end - start) {
            buf[i] = tmp[i + start_sector_read_start];
        }
    }

    fn write_at(&self, offset: usize, buf: &[u8]) {
        let start = offset;
        let end = start + buf.len();

        let start_sector_read_start = start % 512;
        //let end_sector_read_end = end % 512;

        let start_sector_id = start / 512;
        let end_sector_id = (end - 1) / 512;

        let buffer_size = (end_sector_id - start_sector_id + 1) * 512;
        let mut tmp = Vec::new();
        for _ in 0..buffer_size {
            tmp.push(0);
        }
        let tmp = tmp.leak();

        crate::drivers::ahci::read_block(self.hd, start_sector_id as u64, tmp).unwrap();

        for i in 0..(end - start) {
            tmp[i + start_sector_read_start] = buf[i];
        }

        crate::drivers::ahci::write_block(self.hd, start_sector_id as u64, tmp).unwrap();
    }
}
