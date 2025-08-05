use alloc::vec::Vec;

use crate::{
    PhyscialMemoryError, ZodiacError,
    hal::mem::align_down_by_page_size,
    mem::{FRAME_ALLOCATOR, PageSize, PhysicalAddress},
};

pub struct PhysicalMemoryAllocOptions {
    count: usize,
    page_size: PageSize,
    contiguous: bool,
    address: Option<PhysicalAddress>,
}

impl Default for PhysicalMemoryAllocOptions {
    fn default() -> Self {
        Self {
            count: 1,
            page_size: PageSize::Size4K,
            contiguous: false,
            address: None,
        }
    }
}

impl PhysicalMemoryAllocOptions {
    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    pub fn page_size(mut self, page_size: PageSize) -> Self {
        self.page_size = page_size;
        self
    }

    pub fn contiguous(mut self, contiguous: bool) -> Self {
        self.contiguous = contiguous;
        self
    }

    pub(crate) fn address(mut self, address: PhysicalAddress) -> Self {
        self.address = Some(address);
        self
    }
}

impl PhysicalMemoryAllocOptions {
    pub fn allocate(self) -> Result<PhysicalMemory, ZodiacError> {
        if let Some(address) = self.address {
            if !self.page_size.is_aligned(address) || !self.contiguous {
                Err(ZodiacError::InvalidArguments)
            } else {
                Ok(PhysicalMemory::from_start_address(
                    address,
                    self.count,
                    self.page_size,
                ))
            }
        } else {
            PhysicalMemory::new(self.count, self.page_size, self.contiguous)
        }
    }
}

pub struct PhysicalMemory {
    count: usize,
    page_size: PageSize,
    contiguous: bool,
    start_address: Option<PhysicalAddress>,
    frames: Vec<PhysicalAddress>,
}

impl PhysicalMemory {
    fn new(count: usize, page_size: PageSize, contiguous: bool) -> Result<Self, ZodiacError> {
        let one_frame_in_4k = page_size as usize / PageSize::Size4K as usize;
        let start_address = if contiguous {
            Some(
                FRAME_ALLOCATOR
                    .lock()
                    .allocate_frames(count * one_frame_in_4k, one_frame_in_4k)
                    .ok_or(PhyscialMemoryError::AllocateFailed(count))?,
            )
        } else {
            None
        };

        let mut frames = Vec::new();
        if start_address.is_none() {
            for _ in 0..count {
                frames.push(
                    FRAME_ALLOCATOR
                        .lock()
                        .allocate_frames(one_frame_in_4k, one_frame_in_4k)
                        .ok_or(PhyscialMemoryError::AllocateFailed(count))?,
                );
            }
        }

        Ok(Self {
            count,
            page_size,
            contiguous,
            start_address,
            frames,
        })
    }
}

impl PhysicalMemory {
    fn from_start_address(
        start_address: PhysicalAddress,
        count: usize,
        page_size: PageSize,
    ) -> Self {
        Self {
            count,
            page_size,
            contiguous: true,
            start_address: Some(start_address),
            frames: Vec::new(),
        }
    }

    pub fn containing_address(address: PhysicalAddress, count: usize, page_size: PageSize) -> Self {
        let start_address = align_down_by_page_size(address);

        Self::from_start_address(start_address, count, page_size)
    }
}

impl PhysicalMemory {
    pub fn get_start_address_of_frame(&self, id: usize) -> Result<PhysicalAddress, ZodiacError> {
        if id >= self.count() {
            return Err(ZodiacError::InvalidArguments);
        }

        if self.contiguous() {
            Ok(self.start_address.unwrap() + (id * self.page_size as usize))
        } else {
            Ok(*self.frames.get(id).unwrap())
        }
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn page_size(&self) -> PageSize {
        self.page_size
    }

    pub fn contiguous(&self) -> bool {
        self.contiguous
    }
}
