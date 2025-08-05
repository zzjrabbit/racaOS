use core::mem::transmute;

use alloc::sync::Arc;
use object::{
    File, Object, ObjectSegment, SegmentFlags,
    elf::{PF_W, PF_X},
    pe::{IMAGE_SCN_MEM_EXECUTE, IMAGE_SCN_MEM_WRITE},
};
use spin::RwLock;

use crate::{
    ZodiacError,
    hal::mem::*,
    mem::{
        GeneralPageTable, MMUFlags, Page, PageSize, PhysicalMemory, PhysicalMemoryAllocOptions,
        VirtualAddress, convert_physical_to_virtual,
    },
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

    pub fn reader(&self, address: VirtualAddress, len: usize) -> VmReader {
        VmReader {
            address,
            len,
            page_table: self.page_table.clone(),
        }
    }

    pub fn writer(&self, address: VirtualAddress, len: usize) -> VmWriter {
        VmWriter {
            address,
            len,
            page_table: self.page_table.clone(),
        }
    }

    pub fn binary_file_mapper<'a>(&'a self) -> BinaryFileMapper<'a> {
        BinaryFileMapper { vm_space: self }
    }
}

impl VirtualMemorySpace {
    pub fn switch(&self) {
        self.page_table.read().switch();
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
            ) && first_error.is_none()
            {
                first_error = Some(error);
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

pub struct VmReader {
    address: VirtualAddress,
    len: usize,
    page_table: Arc<RwLock<dyn GeneralPageTable>>,
}

impl VmReader {
    pub fn read(&self, buffer: &mut [u8]) -> Result<(), ZodiacError> {
        let mut read = 0usize;

        if self.len != buffer.len() {
            return Err(ZodiacError::InvalidArguments);
        }

        while read < self.len {
            let current_address = self.address + read;

            let (physical_address, _, page_size) =
                self.page_table.write().query(current_address)?;
            let page_offset = page_size.page_offset(current_address);
            let remaining = self.len - read;
            let chunk_size = (page_size as usize - page_offset).min(remaining);

            let virtual_address = convert_physical_to_virtual(physical_address);

            unsafe {
                core::ptr::copy_nonoverlapping(
                    virtual_address as *const u8,
                    buffer[read..read + chunk_size].as_mut_ptr(),
                    chunk_size,
                );
                read += chunk_size;
            }
        }

        Ok(())
    }
}

pub struct VmWriter {
    address: VirtualAddress,
    len: usize,
    page_table: Arc<RwLock<dyn GeneralPageTable>>,
}

impl VmWriter {
    pub fn write(&self, buffer: &[u8]) -> Result<(), ZodiacError> {
        let mut written = 0usize;

        if self.len != buffer.len() {
            return Err(ZodiacError::InvalidArguments);
        }

        while written < self.len {
            let current_address = self.address + written;

            let (physical_address, _, page_size) =
                self.page_table.write().query(current_address)?;
            let page_offset = page_size.page_offset(current_address);
            let remaining = self.len - written;
            let chunk_size = (page_size as usize - page_offset).min(remaining);

            let virtual_address = convert_physical_to_virtual(physical_address);

            unsafe {
                core::ptr::copy_nonoverlapping(
                    buffer[written..written + chunk_size].as_ptr(),
                    virtual_address as *mut u8,
                    chunk_size,
                );
                written += chunk_size;
            }
        }

        Ok(())
    }
}

pub struct BinaryFileMapper<'a> {
    vm_space: &'a VirtualMemorySpace,
}

impl<'a> BinaryFileMapper<'a> {
    pub fn map(&mut self, binary: &[u8]) -> Result<fn() -> !, ZodiacError> {
        let file = File::parse(binary).map_err(|_| ZodiacError::InvalidArguments)?;

        for segment in file.segments() {
            let origin_address = segment.address() as VirtualAddress;
            let length = segment.size() as usize;

            let page_size = PageSize::Size4K;
            let address = page_size.align_down(origin_address);
            let length = page_size.align_up(origin_address + length);

            let physical_memory = PhysicalMemoryAllocOptions::default()
                .count(length / page_size as usize)
                .allocate()?;
            let mut cursor = self.vm_space.cursor(address, page_size)?;

            let mut flags = MMUFlags::READ | MMUFlags::USER;
            let raw_flags = segment.flags();
            match raw_flags {
                SegmentFlags::Elf { p_flags } => {
                    if p_flags & PF_W != 0 {
                        flags |= MMUFlags::WRITE;
                    }
                    if p_flags & PF_X != 0 {
                        flags |= MMUFlags::EXECUTE;
                    }
                }

                SegmentFlags::Coff { characteristics } => {
                    if characteristics & IMAGE_SCN_MEM_WRITE != 0 {
                        flags |= MMUFlags::WRITE;
                    }
                    if characteristics & IMAGE_SCN_MEM_EXECUTE != 0 {
                        flags |= MMUFlags::EXECUTE;
                    }
                }

                _ => return Err(ZodiacError::InvalidArguments),
            }

            if cursor.map(&physical_memory, flags).is_err() {
                cursor.protect(length, flags)?;
            }

            let data = segment.data().map_err(|_| ZodiacError::InvalidArguments)?;

            self.vm_space
                .writer(origin_address, data.len())
                .write(data)
                .map_err(|_| ZodiacError::InvalidArguments)?;
        }

        Ok(unsafe { transmute::<usize, fn() -> !>(file.entry() as usize) })
    }
}
