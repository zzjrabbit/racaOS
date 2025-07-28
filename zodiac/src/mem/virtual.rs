use alloc::sync::Arc;
use spin::RwLock;

use crate::{
    ZodiacError,
    hal::mem::*,
    mem::{GeneralPageTable, MMUFlags, Page, PageSize, PhysicalMemory, VirtualAddress},
};

pub struct VirtualMemorySpace {
    page_table: Arc<RwLock<dyn GeneralPageTable>>,
}

impl VirtualMemorySpace {
    pub fn new_kernel() -> Self {
        Self {
            page_table: kernel_page_table(),
        }
    }

    pub fn new_user() -> Self {
        Self {
            page_table: kernel_page_table().read().deep_copy(),
        }
    }
}

impl VirtualMemorySpace {
    pub fn cursor(
        &self,
        virtual_address: VirtualAddress,
        page_size: PageSize,
    ) -> Result<Cursor, ZodiacError> {
        Cursor::new(self.page_table.clone(), virtual_address, page_size)
    }
}

pub struct Cursor {
    page_table: Arc<RwLock<dyn GeneralPageTable>>,
    virtual_address: VirtualAddress,
    page_size: PageSize,
}

impl Cursor {
    fn new(
        page_table: Arc<RwLock<dyn GeneralPageTable>>,
        virtual_address: VirtualAddress,
        page_size: PageSize,
    ) -> Result<Self, ZodiacError> {
        if !page_size.is_aligned(virtual_address) {
            return Err(ZodiacError::InvalidArguments);
        }
        Ok(Self {
            page_table,
            virtual_address,
            page_size,
        })
    }
}

impl Cursor {
    pub fn map(
        &mut self,
        physical_memory: &PhysicalMemory,
        flags: MMUFlags,
    ) -> Result<(), ZodiacError> {
        let page_size = self.page_size;
        if page_size != physical_memory.page_size() {
            return Err(ZodiacError::InvalidArguments);
        }

        let vaddr = self.virtual_address;
        let page_count = physical_memory.count();
        let mut first_error = None;

        for index in 0..page_count {
            if let Err(error) = self.page_table.write().map(
                Page::new_aligned(vaddr + page_size as usize * index, page_size),
                physical_memory.get_start_address_of_frame(index)?,
                flags,
            ) {
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }
        
        if let Some(error) = first_error {
            return Err(error);
        }

        self.virtual_address += page_size as usize * page_count;

        Ok(())
    }

    pub fn unmap(&mut self, len: usize) -> Result<(), ZodiacError> {
        let page_size = self.page_size;
        let vaddr = self.virtual_address;

        self.page_table
            .write()
            .unmap_cont(vaddr, page_size.align_up(len))?;

        self.virtual_address += len;

        Ok(())
    }

    pub fn protect(&mut self, len: usize, flags: MMUFlags) -> Result<(), ZodiacError> {
        let page_size = self.page_size;
        let vaddr = self.virtual_address;

        let len = page_size.align_up(len);
        let page_count = len / page_size as usize;

        for index in 0..page_count {
            self.page_table
                .write()
                .update(vaddr + page_size as usize * index, flags)?;
        }

        self.virtual_address += len;

        Ok(())
    }
}

impl Cursor {
    pub fn jump_to(&mut self, virtual_address: VirtualAddress) -> Result<(), ZodiacError> {
        if !self.page_size.is_aligned(virtual_address) {
            return Err(ZodiacError::InvalidArguments);
        }

        self.virtual_address = virtual_address;
        Ok(())
    }
}
