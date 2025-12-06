use crate::{
    PhyscialMemoryError, ZodiacError,
    mem::{
        FRAME_ALLOCATOR, PageSize, PhysicalAddress, VmSpace, convert_physical_to_virtual,
        vm_space::{VmReader, VmWriter},
    },
};

pub struct PhysicalMemoryAllocOptions {
    count: usize,
}

impl PhysicalMemoryAllocOptions {
    pub const fn new() -> Self {
        Self { count: 1 }
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
}

impl PhysicalMemoryAllocOptions {
    /// Allocate physical memory frames with the specified options.
    pub fn allocate(self) -> Result<PhysicalMemory, ZodiacError> {
        PhysicalMemory::new(self.count)
    }
}

#[derive(Debug)]
pub struct PhysicalMemory {
    count: usize,
    start_address: PhysicalAddress,
}

impl PhysicalMemory {
    fn new(count: usize) -> Result<Self, ZodiacError> {
        let start_address = FRAME_ALLOCATOR
            .lock()
            .allocate_frames(count)
            .ok_or(PhyscialMemoryError::AllocateFailed(count))?;

        Ok(Self {
            count,
            start_address,
        })
    }
}

impl PhysicalMemory {
    pub fn from_start_address(start_address: PhysicalAddress, count: usize) -> Self {
        Self {
            count,
            start_address,
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

        Ok(self.start_address + (id * PageSize::Size4K as usize))
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

impl PhysicalMemory {
    pub fn reader(&self, offset: usize, size: usize) -> VmReader {
        VmSpace::kernel().reader(
            convert_physical_to_virtual(self.start_address) + offset,
            size,
        )
    }

    pub fn writer(&self, offset: usize, size: usize) -> VmWriter {
        VmSpace::kernel().writer(
            convert_physical_to_virtual(self.start_address) + offset,
            size,
        )
    }

    pub fn zero(&self) -> Result<(), ZodiacError> {
        let page_size = PageSize::Size4K;

        let zeros = alloc::vec![0; page_size as usize];
        for id in 0..self.count {
            VmSpace::kernel()
                .writer(
                    convert_physical_to_virtual(self.start_address) + id * page_size as usize,
                    page_size as usize,
                )
                .write_bytes(&zeros)?;
        }
        Ok(())
    }
}
