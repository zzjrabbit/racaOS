use alloc::vec::Vec;

use crate::{
    PhyscialMemoryError, ZodiacError,
    mem::{FRAME_ALLOCATOR, PageSize, PhysicalAddress, convert_physical_to_virtual},
};

/// Options for allocating physical memory.
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
    /// Set the number of frames to allocate.
    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    /// Set the page size for the allocated frames.
    pub fn page_size(mut self, page_size: PageSize) -> Self {
        self.page_size = page_size;
        self
    }

    /// Set whether the allocated frames should be contiguous.
    pub fn contiguous(mut self, contiguous: bool) -> Self {
        self.contiguous = contiguous;
        self
    }

    /// Set the starting address for the allocated frames.
    /// This is only useful when allocating contiguous frames.
    pub(crate) fn address(mut self, address: PhysicalAddress) -> Self {
        self.address = Some(address);
        self
    }
}

impl PhysicalMemoryAllocOptions {
    /// Allocate physical memory frames with the specified options.
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

/// Manages multiple physical memory frames.
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
    pub fn from_start_address(
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
        let start_address = page_size.align_down(address);

        Self::from_start_address(start_address, count, page_size)
    }

    pub fn deallocate(&self) {
        for id in 0..self.count() {
            let start_address = self.get_start_address_of_frame(id).unwrap();
            FRAME_ALLOCATOR.lock().deallocate_frames(start_address, 1);
        }
    }
}

impl PhysicalMemory {
    pub fn as_slice(&self, id: usize) -> Result<&[u8], ZodiacError> {
        let paddr = self.get_start_address_of_frame(id)?;
        let vaddr = convert_physical_to_virtual(paddr);

        Ok(unsafe { core::slice::from_raw_parts(vaddr as *const u8, self.page_size as usize) })
    }

    pub fn as_mut_slice(&mut self, id: usize) -> Result<&mut [u8], ZodiacError> {
        let paddr = self.get_start_address_of_frame(id)?;
        let vaddr = convert_physical_to_virtual(paddr);

        Ok(unsafe { core::slice::from_raw_parts_mut(vaddr as *mut u8, self.page_size as usize) })
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
