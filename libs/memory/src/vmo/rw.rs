use ostd::{
    Error, Pod,
    mm::{PAGE_SIZE, VmIo},
};

use crate::{Vmo, align_down_by_page_size};

impl Vmo {
    pub fn read_bytes(&self, offset: usize, buffer: &mut [u8]) -> Result<(), Error> {
        if self.is_iomem() {
            let (iomem, base_offset) = self.into_iomem().unwrap();
            iomem.read_bytes(offset - base_offset, buffer)?;
        } else {
            let mut read = 0;
            while read < buffer.len() {
                let current_offset = offset + read;
                let page_offset = current_offset % PAGE_SIZE;
                let remaining = buffer.len() - read;
                let chunk_size = (PAGE_SIZE - page_offset).min(remaining);

                let aligned_offset = align_down_by_page_size(current_offset);
                let (_, frame) = self.into_ram(aligned_offset)?.unwrap();
                frame.read_bytes(page_offset, &mut buffer[read..read + chunk_size])?;
                read += chunk_size;
            }
        }

        Ok(())
    }

    pub fn write_bytes(&self, offset: usize, buffer: &[u8]) -> Result<(), Error> {
        if self.is_iomem() {
            let (iomem, base_offset) = self.into_iomem().unwrap();
            iomem.write_bytes(offset - base_offset, buffer)?;
        } else {
            let mut written = 0;
            while written < buffer.len() {
                let current_offset = offset + written;
                let page_offset = current_offset % PAGE_SIZE;
                let remaining = buffer.len() - written;
                let chunk_size = (PAGE_SIZE - page_offset).min(remaining);

                let aligned_offset = align_down_by_page_size(current_offset);
                let (_, frame) = self.into_ram(aligned_offset)?.unwrap();
                frame.write_bytes(page_offset, &buffer[written..written + chunk_size])?;
                written += chunk_size;
            }
        }

        Ok(())
    }

    pub fn read_val<T: Pod>(&self, offset: usize) -> Result<T, Error> {
        let mut value = T::new_uninit();
        let buffer = value.as_bytes_mut();
        self.read_bytes(offset, buffer)?;
        Ok(value)
    }

    pub fn write_val<T: Pod>(&self, offset: usize, value: &T) -> Result<(), Error> {
        let buffer = value.as_bytes();
        self.write_bytes(offset, buffer)?;
        Ok(())
    }
}
