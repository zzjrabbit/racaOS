use ostd::{
    mm::{VmIo, PAGE_SIZE},
    Error as OstdError, Pod,
};

use crate::mem::{align_down_by_page_size, Vmar};

impl Vmar {
    pub fn read_val<T: Pod>(&self, address: usize) -> Result<T, OstdError> {
        let mut buffer = alloc::vec![0u8; core::mem::size_of::<T>()];
        self.read(address, &mut buffer)?;
        Ok(T::from_bytes(&buffer))
    }

    pub fn write_val<T: Pod>(&self, address: usize, value: &T) -> Result<(), OstdError> {
        let buffer = value.as_bytes();
        self.write(address, buffer)?;
        Ok(())
    }

    pub fn read(&self, address: usize, buffer: &mut [u8]) -> Result<(), OstdError> {
        let mut read: usize = 0;

        while read < buffer.len() {
            let current_address = address + read;
            let page_offset = current_address % PAGE_SIZE;
            let remaining = buffer.len() - read;
            let chunk_size = (PAGE_SIZE - page_offset).min(remaining);

            let page_address = align_down_by_page_size(current_address);
            let frame = self
                .inner
                .read()
                .vm_mappings
                .iter()
                .find(|mapping| mapping.contains(page_address))
                .map(|mapping| {
                    mapping.frames()[(page_address - mapping.start()) / PAGE_SIZE].clone()
                })
                .unwrap();

            frame.read_bytes(page_offset, &mut buffer[read..read + chunk_size])?;
            read += chunk_size;
        }

        Ok(())
    }

    pub fn write(&self, address: usize, buffer: &[u8]) -> Result<(), OstdError> {
        let mut written: usize = 0;

        while written < buffer.len() {
            let current_address = address + written;
            let page_offset = current_address % PAGE_SIZE;
            let remaining = buffer.len() - written;
            let chunk_size = (PAGE_SIZE - page_offset).min(remaining);

            let page_address = align_down_by_page_size(current_address);
            let frame = self
                .inner
                .read()
                .vm_mappings
                .iter()
                .find(|mapping| mapping.contains(page_address))
                .map(|mapping| {
                    mapping.frames()[(page_address - mapping.start()) / PAGE_SIZE].clone()
                })
                .unwrap();

            frame.write_bytes(page_offset, &buffer[written..written + chunk_size])?;
            written += chunk_size;
        }

        Ok(())
    }
}
