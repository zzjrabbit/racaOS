use alloc::{ffi::CString, vec::Vec};
use errors::{Errno, Result};
use mostd::mem::{Pod, VirtualAddress};

use crate::Vmar;

impl Vmar {
    pub fn read_val<T: Pod>(&self, address: usize) -> Result<T> {
        let mut buffer = T::new_uninit();
        self.vm_space
            .reader(address, size_of::<T>())
            .read(&mut buffer)?;
        Ok(buffer)
    }

    pub fn write_val<T: Pod>(&self, address: usize, value: &T) -> Result<()> {
        self.vm_space.writer(address, size_of::<T>()).write(value)?;
        Ok(())
    }

    pub fn read(&self, address: usize, buffer: &mut [u8]) -> Result<()> {
        self.vm_space
            .reader(address, buffer.len())
            .read_bytes(buffer)?;
        Ok(())
    }

    pub fn write(&self, address: usize, buffer: &[u8]) -> Result<()> {
        self.vm_space
            .writer(address, buffer.len())
            .write_bytes(buffer)?;
        Ok(())
    }
}

impl Vmar {
    pub fn read_cstring(
        &self,
        address: VirtualAddress,
        max_string_len: Option<usize>,
    ) -> Result<CString> {
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
