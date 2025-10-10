use alloc::vec::Vec;
use ostd::{Error, mm::{FrameAllocOptions, PAGE_SIZE, PageFlags, PageProperty, UFrame, Vaddr, VmIo}};

#[derive(Debug)]
pub struct VmMapping {
    frames: Vec<UFrame>,
    start: Vaddr,
    size: usize,
    prop: PageProperty,
    mapped: bool,
}

impl VmMapping {
    pub fn new(frames: Vec<UFrame>, start: Vaddr, size: usize, prop: PageProperty) -> Self {
        VmMapping {
            frames,
            start,
            size,
            prop,
            mapped: false,
        }
    }
}

#[allow(dead_code)]
impl VmMapping {
    pub fn frames(&self) -> &[UFrame] {
        &self.frames
    }

    pub fn start(&self) -> Vaddr {
        self.start
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn prop(&self) -> PageProperty {
        self.prop
    }

    pub fn set_prop(&mut self, prop: PageProperty) {
        self.prop = prop;
    }

    pub fn overlaps(&self, other: &VmMapping) -> bool {
        self.start <= other.start + other.size && other.start < self.start + self.size
    }

    pub fn contains_range(&self, start: Vaddr, size: usize) -> bool {
        self.start <= start && start + size <= self.start + self.size
    }

    pub fn contains(&self, addr: Vaddr) -> bool {
        self.start <= addr && addr < self.start + self.size
    }

    pub fn mapped(&self) -> bool {
        self.mapped
    }

    pub fn map(&mut self) {
        self.mapped = true;
    }
}

impl VmMapping {
    pub fn clone(&self) -> Result<Self, Error> {
        let frames = if self.prop().flags.contains(PageFlags::W) {
            let mut frames = Vec::new();
            let mut data = alloc::vec![0u8; PAGE_SIZE];
            for source in self.frames().iter() {
                let dest = FrameAllocOptions::new().alloc_frame()?;
                
                source.read_bytes(0, &mut data)?;
                dest.write_bytes(0, &data)?;
                
                frames.push(dest.into());
            }
            frames
        } else {
            self.frames.clone()
        };
        
        Ok(Self::new(frames, self.start, self.size, self.prop))
    }
}
