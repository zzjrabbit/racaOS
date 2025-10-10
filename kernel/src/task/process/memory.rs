use alloc::{sync::Arc, vec::Vec};
use ostd::{
    mm::{PageProperty, Vaddr},
    sync::RwLock,
    Error as OstdError,
};

use crate::mem::Vmar;

/// The base of kernel address space.
pub const KERNEL_ASPACE_BASE: usize = 0xffff_ff80_0000_0000;
/// The base of user address space.
pub const USER_ASPACE_BASE: usize = 0x0000_0001_0000_0000;
/// The size of user address space.
pub const USER_ASPACE_SIZE: usize = KERNEL_ASPACE_BASE - USER_ASPACE_BASE;

#[derive(Debug)]
pub struct MemoryInfo {
    vmar: Arc<Vmar>,
    inner: RwLock<MemoryInfoInner>,
}

#[derive(Debug, Clone)]
struct MemoryInfoInner {
    free_memory_space: Vec<MemoryRegion>,
    allocated: Vec<MemoryRegion>,
    unused_regions: Vec<(MemoryRegion, PageProperty)>,
}

impl MemoryInfo {
    pub fn new(vmar: Arc<Vmar>) -> Self {
        MemoryInfo {
            vmar,
            inner: RwLock::new(MemoryInfoInner {
                free_memory_space: alloc::vec![MemoryRegion::new(
                    USER_ASPACE_BASE,
                    USER_ASPACE_SIZE
                )],
                allocated: Vec::new(),
                unused_regions: Vec::new(),
            }),
        }
    }

    pub fn deep_clone(&self) -> Self {
        MemoryInfo {
            vmar: self.vmar.deep_clone().unwrap(),
            inner: RwLock::new(self.inner.read().clone()),
        }
    }
}

#[allow(dead_code)]
impl MemoryInfo {
    pub fn vmar(&self) -> Arc<Vmar> {
        self.vmar.clone()
    }

    pub fn with_unused_regions<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Vec<(MemoryRegion, PageProperty)>) -> R,
    {
        f(&self.inner.read().unused_regions)
    }

    pub fn with_unused_regions_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut Vec<(MemoryRegion, PageProperty)>) -> R,
    {
        f(&mut self.inner.write().unused_regions)
    }
}

impl MemoryInfo {
    pub fn allocate(&self, len: usize) -> Result<MemoryRegion, OstdError> {
        let mut inner = self.inner.write();

        let mut region = None;

        for free_region in inner.free_memory_space.iter_mut() {
            if free_region.end - free_region.start >= len {
                let allocated_region = MemoryRegion::new(free_region.start, len);
                free_region.start = allocated_region.end;
                region = Some(allocated_region);
                break;
            }
        }

        if let Some(region) = region {
            inner.allocated.push(region);
        }

        region.ok_or(OstdError::NoMemory)
    }

    pub fn allocate_at(&self, address: Vaddr, len: usize) -> Result<MemoryRegion, OstdError> {
        let mut inner = self.inner.write();

        let required_region = MemoryRegion::new(address, len);

        for mapped in inner.allocated.iter() {
            if mapped.overlap(&required_region) {
                return Err(OstdError::NoMemory);
            }
        }

        inner.allocated.push(required_region);

        let mut new_region = None;

        for region in inner.free_memory_space.iter_mut() {
            if region.overlap(&required_region) {
                if region.start == required_region.start {
                    region.end = required_region.end;
                } else if region.end == required_region.end {
                    region.start = required_region.start;
                } else {
                    region.end = required_region.start;
                    new_region = Some(MemoryRegion::new(
                        required_region.end,
                        region.end - required_region.end,
                    ));
                }
            }
        }

        if let Some(region) = new_region {
            inner.free_memory_space.push(region);
        }

        Ok(required_region)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    start: usize,
    end: usize,
}

#[allow(dead_code)]
impl MemoryRegion {
    pub fn new(start: usize, len: usize) -> Self {
        MemoryRegion {
            start,
            end: start + len,
        }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn start_address(&self) -> usize {
        self.start
    }

    pub fn end_address(&self) -> usize {
        self.end
    }

    pub fn contains(&self, addr: usize) -> bool {
        self.start <= addr && addr < self.end
    }

    pub fn overlap(&self, other: &MemoryRegion) -> bool {
        self.start <= other.start && self.end >= other.end
    }
}
