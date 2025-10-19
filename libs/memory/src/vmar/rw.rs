use alloc::{ffi::CString, vec::Vec};
use errors::{Errno, Result};
use ostd::{Pod, mm::Vaddr};

use crate::Vmar;

impl Vmar {
    pub fn read_val<T: Pod>(&self, address: usize) -> Result<T> {
        let mut buffer = alloc::vec![0u8; core::mem::size_of::<T>()];
        self.read(address, &mut buffer)?;
        Ok(T::from_bytes(&buffer))
    }

    pub fn write_val<T: Pod>(&self, address: usize, value: &T) -> Result<()> {
        let buffer = value.as_bytes();
        self.write(address, buffer)?;
        Ok(())
    }

    pub fn read(&self, address: usize, buffer: &mut [u8]) -> Result<()> {
        let mut read: usize = 0;

        while read < buffer.len() {
            let current_address = address + read;

            let (mapping_start, mapping_size, vmo) = self
                .inner
                .read()
                .vm_mappings
                .iter()
                .find(|mapping| mapping.contains(current_address))
                .map(|mapping| (mapping.start(), mapping.size(), mapping.vmo().clone()))
                .unwrap();

            let remaining = buffer.len() - read;
            let chunk_size = mapping_size.min(remaining);

            vmo.read_bytes(
                current_address - mapping_start,
                &mut buffer[read..read + chunk_size],
            )?;
            read += chunk_size;
        }

        Ok(())
    }

    pub fn write(&self, address: usize, buffer: &[u8]) -> Result<()> {
        let mut written: usize = 0;

        while written < buffer.len() {
            let current_address = address + written;

            let (mapping_start, mapping_size, vmo) = self
                .inner
                .read()
                .vm_mappings
                .iter()
                .find(|mapping| mapping.contains(current_address))
                .map(|mapping| (mapping.start(), mapping.size(), mapping.vmo().clone()))
                .unwrap();

            let remaining = buffer.len() - written;
            let chunk_size = mapping_size.min(remaining);

            vmo.write_bytes(
                current_address - mapping_start,
                &buffer[written..written + chunk_size],
            )?;
            written += chunk_size;
        }

        Ok(())
    }
}

impl Vmar {
    pub fn read_cstring(&self, address: Vaddr, max_string_len: Option<usize>) -> Result<CString> {
        let mut buffer = Vec::new();
        let mut current_address = address;

        loop {
            if current_address - address == max_string_len.unwrap_or(usize::MAX) {
                return Err(Errno::E2BIG.no_message());
            }
            
            let byte: u8 = self.read_val(current_address)?;
            if byte == 0 {
                return CString::new(buffer).map_err(|_| Errno::EINVAL.no_message());
            }
            buffer.push(byte);
            current_address += 1;
        }
    }
}
