use alloc::vec::Vec;

use crate::{
    PhyscialMemoryError, ZodiacError,
    mem::{FRAME_ALLOCATOR, PageSize, PhysicalAddress, convert_physical_to_virtual},
};

pub struct PhysicalMemoryAllocOptions {
    count: usize,
    continuous: bool,
    address: Option<PhysicalAddress>,
}

impl PhysicalMemoryAllocOptions {
    pub const fn new() -> Self {
        Self {
            count: 1,
            continuous: false,
            address: None,
        }
    }
}

impl Default for PhysicalMemoryAllocOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicalMemoryAllocOptions {
    /// Set the number of frames to allocate.
    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    /// Set whether the allocated frames should be contiguous.
    pub fn continuous(mut self, continuous: bool) -> Self {
        self.continuous = continuous;
        self
    }

    /// Set the starting address for the allocated frames.
    /// This is only useful when allocating continuous frames.
    pub(crate) fn address(mut self, address: PhysicalAddress) -> Self {
        self.address = Some(address);
        self
    }
}

impl PhysicalMemoryAllocOptions {
    /// Allocate physical memory frames with the specified options.
    pub fn allocate(self) -> Result<PhysicalMemory, ZodiacError> {
        if let Some(address) = self.address {
            if !PageSize::Size4K.is_aligned(address) || !self.continuous {
                Err(ZodiacError::InvalidArguments)
            } else {
                Ok(PhysicalMemory::from_start_address(address, self.count))
            }
        } else {
            PhysicalMemory::new(self.count, self.continuous)
        }
    }
}

pub struct PhysicalMemory {
    count: usize,
    continuous: bool,
    start_address: Option<PhysicalAddress>,
    frames: Vec<PhysicalAddress>,
}

impl PhysicalMemory {
    fn new(count: usize, continuous: bool) -> Result<Self, ZodiacError> {
        let start_address = if continuous {
            Some(
                FRAME_ALLOCATOR
                    .lock()
                    .allocate_frames(count)
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
                        .allocate_frames(1)
                        .ok_or(PhyscialMemoryError::AllocateFailed(count))?,
                );
            }
        }

        Ok(Self {
            count,
            continuous,
            start_address,
            frames,
        })
    }
}

impl PhysicalMemory {
    pub fn from_start_address(start_address: PhysicalAddress, count: usize) -> Self {
        Self {
            count,
            continuous: true,
            start_address: Some(start_address),
            frames: Vec::new(),
        }
    }

    pub fn containing_address(address: PhysicalAddress, count: usize) -> Self {
        let start_address = PageSize::Size4K.align_down(address);

        Self::from_start_address(start_address, count)
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

        Ok(unsafe { core::slice::from_raw_parts(vaddr as *const u8, PageSize::Size4K as usize) })
    }

    pub fn as_mut_slice(&mut self, id: usize) -> Result<&mut [u8], ZodiacError> {
        let paddr = self.get_start_address_of_frame(id)?;
        let vaddr = convert_physical_to_virtual(paddr);

        Ok(unsafe { core::slice::from_raw_parts_mut(vaddr as *mut u8, PageSize::Size4K as usize) })
    }
}

impl PhysicalMemory {
    pub fn get_start_address_of_frame(&self, id: usize) -> Result<PhysicalAddress, ZodiacError> {
        if id >= self.count() {
            return Err(ZodiacError::InvalidArguments);
        }

        if self.continuous() {
            Ok(self.start_address.unwrap() + (id * PageSize::Size4K as usize))
        } else {
            Ok(*self.frames.get(id).unwrap())
        }
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn continuous(&self) -> bool {
        self.continuous
    }
}
