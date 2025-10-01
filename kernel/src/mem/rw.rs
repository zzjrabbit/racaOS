use ostd::{
    mm::{vm_space::VmQueriedItem, VmIo, VmSpace, PAGE_SIZE},
    task::disable_preempt,
    Error as OstdError, Pod,
};

use crate::mem::align_down_by_page_size;

#[allow(dead_code)]
pub trait VmReadWrite {
    fn read(&self, address: usize, buffer: &mut [u8]) -> Result<(), OstdError>;
    fn write(&self, address: usize, buffer: &[u8]) -> Result<(), OstdError>;

    fn read_val<T: Pod>(&self, address: usize) -> Result<T, OstdError> {
        let mut buffer = alloc::vec![0u8; core::mem::size_of::<T>()];
        self.read(address, &mut buffer)?;
        Ok(T::from_bytes(&buffer))
    }

    fn write_val<T: Pod>(&self, address: usize, value: &T) -> Result<(), OstdError> {
        let buffer = value.as_bytes();
        self.write(address, &buffer)?;
        Ok(())
    }
}

impl VmReadWrite for VmSpace {
    fn read(&self, address: usize, buffer: &mut [u8]) -> Result<(), OstdError> {
        let guard = disable_preempt();

        let mut read: usize = 0;

        while read < buffer.len() {
            let current_address = address + read;
            let page_offset = current_address % PAGE_SIZE;
            let remaining = buffer.len() - read;
            let chunk_size = (PAGE_SIZE - page_offset).min(remaining) as usize;

            let page_address = align_down_by_page_size(current_address);
            let (_, Some(VmQueriedItem::MappedRam { frame, prop: _ })) = self
                .cursor_mut(&guard, &(page_address..page_address + PAGE_SIZE))?
                .query()?
            else {
                return Err(OstdError::InvalidArgs);
            };

            frame.read_bytes(page_offset, &mut buffer[read..read + chunk_size])?;
            read += chunk_size;
        }

        Ok(())
    }

    fn write(&self, address: usize, buffer: &[u8]) -> Result<(), OstdError> {
        let guard = disable_preempt();

        let mut written: usize = 0;

        while written < buffer.len() {
            let current_address = address + written;
            let page_offset = current_address % PAGE_SIZE;
            let remaining = buffer.len() - written;
            let chunk_size = (PAGE_SIZE - page_offset).min(remaining) as usize;

            let page_address = align_down_by_page_size(current_address);
            let (_, Some(VmQueriedItem::MappedRam { frame, prop: _ })) = self
                .cursor_mut(&guard, &(page_address..page_address + PAGE_SIZE))?
                .query()?
            else {
                return Err(OstdError::InvalidArgs);
            };

            frame.write_bytes(page_offset, &buffer[written..written + chunk_size])?;
            written += chunk_size;
        }

        Ok(())
    }
}
