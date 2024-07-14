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

pub type FlushFunction = fn(block_id: usize, cache: &mut [u8]);

pub struct CacheManager<CacheS: Cache> {
    caches: BTreeMap<usize, CacheS>,
}

impl<C: Cache> CacheManager<C> {
    pub const fn new() -> Self {
        Self {
            caches: BTreeMap::new(),
        }
    }

    pub fn add_cache(&mut self, block_id: usize, cache: C) {
        self.caches.insert(block_id, cache);
    }

    pub fn flush_cache(&mut self, func: FlushFunction) {
        for (block_id, cache) in self.caches.iter_mut() {
            func(*block_id, cache.get_buffer());
        }
    }
}
