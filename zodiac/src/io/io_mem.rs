use core::ops::Range;

use alloc::sync::Arc;

use crate::{
    ZodiacError,
    mem::{Pod, VirtualAddress, VmSpace, convert_physical_to_virtual},
};

#[derive(Debug)]
pub struct IoMem {
    start_address: VirtualAddress,
    size: usize,
    vm_space: Arc<VmSpace>,
}

impl IoMem {
    pub fn acquire(range: Range<usize>) -> Result<Arc<Self>, ZodiacError> {
        let start = range.start;
        let size = range.end - range.start;

        let start_address = convert_physical_to_virtual(start);
        log::info!("start address: {:x}", start_address);

        Ok(Arc::new(IoMem {
            start_address,
            size,
            vm_space: VmSpace::kernel(),
        }))
    }
}

impl IoMem {
    pub fn start_address(&self) -> usize {
        self.start_address
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl IoMem {
    pub fn read_bytes(&self, offset: usize, buffer: &mut [u8]) -> Result<(), ZodiacError> {
        let address = self.start_address() + offset;
        self.vm_space
            .reader(address, buffer.len())
            .read_bytes(buffer)
    }

    pub fn read<T: Pod>(&self, offset: usize, value: &mut T) -> Result<(), ZodiacError> {
        let address = self.start_address() + offset;
        self.vm_space.reader(address, size_of::<T>()).read(value)
    }

    pub fn write_bytes(&self, offset: usize, buffer: &[u8]) -> Result<(), ZodiacError> {
        let address = self.start_address() + offset;
        self.vm_space
            .writer(address, buffer.len())
            .write_bytes(buffer)
    }

    pub fn write<T: Pod>(&self, offset: usize, value: &T) -> Result<(), ZodiacError> {
        let address = self.start_address() + offset;
        self.vm_space.writer(address, size_of::<T>()).write(value)
    }
}
