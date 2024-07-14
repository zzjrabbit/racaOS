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

    fn mount(&self, _node: crate::fs::vfs::inode::InodeRef, _name: alloc::string::String) {
        unimplemented!()
    }

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

        /*let mut tmp = [0; 512];
        crate::drivers::ahci::read_block(self.hd, part_read_sectors.0 as u64, &mut tmp);
        for i in (512 - start_part_size - 1)..512 {
            buf[i - (512 - start_part_size - 1)] = tmp[i];
        }

        crate::drivers::ahci::read_block(self.hd, part_read_sectors.1 as u64, &mut tmp);
        for i in 0..end_part_size {
            buf[i + end_sector * 512 + start_part_size] = tmp[i];
        }*/
    }
}
