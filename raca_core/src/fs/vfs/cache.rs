use alloc::{collections::BTreeMap, vec::Vec};

pub trait Cache {
    const SIZE: usize;

    fn new() -> Self;
    fn get_buffer(&mut self) -> &mut [u8];
}

pub struct Cache512B {
    buffer: Vec<u8>,
}

impl Cache for Cache512B {
    const SIZE: usize = 512;

    fn new() -> Self {
        let mut buffer = Vec::new();
        for _ in 0..Self::SIZE {
            buffer.push(0);
        }
        Self { buffer }
    }

    fn get_buffer(&mut self) -> &mut [u8] {
        self.buffer.as_mut_slice()
    }
}

impl Drop for Cache512B {
    fn drop(&mut self) {}
}

pub trait BlockDeviceInterface {
    fn read(&self, block_id: usize, buf: &mut [u8]);
    fn write(&self, block_id: usize, buf: &[u8]);
}

pub struct CacheManager<CacheS: Cache, B: BlockDeviceInterface> {
    caches: BTreeMap<usize, CacheS>,
    block_device: B,
}

impl<C: Cache, B: BlockDeviceInterface> CacheManager<C, B> {
    pub const fn new(block_device: B) -> Self {
        Self {
            caches: BTreeMap::new(),
            block_device,
        }
    }

    fn has_cache_on(&self, block: usize) -> bool {
        self.caches.contains_key(&block)
    }

    fn add_cache(&mut self, block_id: usize, cache: C) {
        self.caches.insert(block_id, cache);
    }

    pub fn flush_cache(&mut self) {
        for (block_id, cache) in self.caches.iter_mut() {
            self.block_device.write(*block_id, cache.get_buffer());
        }
    }

    pub fn read_from_cache(&mut self, start_block: usize, buf: &mut [u8]) {
        let block_num = buf.len() / 512;

        for block_id in start_block..(start_block + block_num) {
            if !self.has_cache_on(block_id) {
                let mut new_cache = C::new();
                self.block_device.read(block_id, new_cache.get_buffer());
                self.add_cache(block_id, new_cache);
            }
            let cache = self.caches.get_mut(&block_id).unwrap();

            let start = (block_id - start_block) * 512;
            buf[start..start + 512].copy_from_slice(cache.get_buffer());
        }
    }

    pub fn write_to_cache(&mut self, start_block: usize, buf: &[u8]) {
        let block_num = buf.len() / 512;
        for block_id in start_block..(start_block + block_num) {
            if !self.has_cache_on(block_id) {
                let mut new_cache = C::new();
                self.block_device.read(block_id, new_cache.get_buffer());
                self.add_cache(block_id, new_cache);
            }

            let cache = self.caches.get_mut(&block_id).unwrap();

            let start = (block_id - start_block) * 512;

            cache.get_buffer().copy_from_slice(&buf[start..start + 512]);
        }
    }
}
