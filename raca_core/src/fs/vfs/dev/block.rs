use alloc::{string::String, vec::Vec};
use framework::ref_to_mut;

use crate::{
    drivers::block::HD_LIST,
    fs::vfs::{
        cache::{BlockDeviceInterface, Cache512B, CacheManager},
        inode::Inode,
    },
};

struct BlockDevice {
    id: usize,
}

impl BlockDevice {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}

impl BlockDeviceInterface for BlockDevice {
    fn read(&self, block_id: usize, buf: &mut [u8]) {
        HD_LIST.lock()[self.id].read_block(block_id, buf);
    }

    fn write(&self, block_id: usize, buf: &[u8]) {
        HD_LIST.lock()[self.id].write_block(block_id, buf);
    }
}

pub struct BlockInode {
    hd: usize,
    cache_manager: CacheManager<Cache512B, BlockDevice>,
    path: String,
}

impl BlockInode {
    pub fn new(hd: usize) -> Self {
        Self {
            hd,
            cache_manager: CacheManager::new(BlockDevice::new(hd)),
            path: String::new(),
        }
    }
}

impl Inode for BlockInode {
    fn when_mounted(
        &mut self,
        path: alloc::string::String,
        _father: Option<crate::fs::vfs::inode::InodeRef>,
    ) {
        self.path.clear();
        self.path.push_str(path.as_str());
    }

    fn when_umounted(&mut self) {}

    fn get_path(&self) -> alloc::string::String {
        self.path.clone()
    }

    fn size(&self) -> usize {
        HD_LIST.lock()[self.hd].get_size()
    }

    fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let start = offset;
        let end = start + buf.len();

        let start_sector_read_start = start % 512;

        let start_sector_id = start / 512;
        let end_sector_id = (end - 1) / 512;

        let buffer_size = (end_sector_id - start_sector_id + 1) * 512;
        let mut tmp = Vec::new();
        for _ in 0..buffer_size {
            tmp.push(0);
        }
        let tmp = tmp.leak();

        ref_to_mut(self)
            .cache_manager
            .read_from_cache(start_sector_id, tmp);

        for i in 0..(end - start) {
            buf[i] = tmp[i + start_sector_read_start];
        }
        buf.len()
    }

    fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let start = offset;
        let end = start + buf.len();

        let start_sector_read_start = start % 512;

        let start_sector_id = start / 512;
        let end_sector_id = (end - 1) / 512;

        let buffer_size = (end_sector_id - start_sector_id + 1) * 512;
        let mut tmp = Vec::new();
        for _ in 0..buffer_size {
            tmp.push(0);
        }
        let tmp = tmp.leak();

        ref_to_mut(self)
            .cache_manager
            .read_from_cache(start_sector_id, tmp);

        for i in 0..(end - start) {
            tmp[i + start_sector_read_start] = buf[i];
        }

        ref_to_mut(self)
            .cache_manager
            .write_to_cache(start_sector_id, tmp);

        ref_to_mut(self).cache_manager.flush_cache();

        buf.len()
    }
}
