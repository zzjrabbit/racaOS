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

/// A structure to manage virtual memory space.
/// Most interfaces are implemented in other structures.
/// But you must construct them with this structure.
pub struct VirtualMemorySpace {
    page_table: Arc<RwLock<dyn GeneralPageTable>>,
}

impl VirtualMemorySpace {
    /// Create a kernel virtual memory space.
    /// This is not a copy, but a direct reference to the kernel space.
    pub fn new_kernel() -> Self {
        Self {
            page_table: kernel_page_table(),
        }
    }

    /// Create a new user virtual memory space.
    /// This is a copy of kernel virtual memory space.
    pub fn new_user() -> Self {
        Self {
            page_table: kernel_page_table().read().deep_copy(),
        }
    }

    /// Deep copy this virtual memory space.
    /// Good choice for fork syscall.
    pub fn deep_copy(&self) -> Self {
        Self {
            page_table: self.page_table.read().deep_copy(),
        }
    }
}

impl VirtualMemorySpace {
    /// Create a cursor at the given virtual address with the given page size.
    /// So that you can map, unmap and change the flags of virtual memory regions.
    pub fn cursor(
        &self,
        virtual_address: VirtualAddress,
        page_size: PageSize,
    ) -> Result<Cursor, ZodiacError> {
        Cursor::new(self.page_table.clone(), virtual_address, page_size)
    }

    /// Create a reader to help you read data from the virtual memory space.
    /// See more at `VmReader`.
    pub fn reader(&self, address: VirtualAddress, len: usize) -> VmReader {
        VmReader {
            address,
            len,
            page_table: self.page_table.clone(),
        }
    }

    /// Create a writer to help you write data into the virtual memory space.
    /// See more at `VmWriter`.
    pub fn writer(&self, address: VirtualAddress, len: usize) -> VmWriter {
        VmWriter {
            address,
            len,
            page_table: self.page_table.clone(),
        }
    }

    /// Create a binary file mapper to map binary files in to the virtual memory space.
    /// See more at `BinaryFileMapper`.
    pub fn binary_file_mapper<'a>(&'a self) -> BinaryFileMapper<'a> {
        BinaryFileMapper { vm_space: self }
    }
}

impl VirtualMemorySpace {
    // Switches to this virtual memory space.
    // Might cause Page Fault if you are not carefull.
    pub fn switch(&self) {
        self.page_table.read().switch();
    }
}

/// An interface to map, unmap and change flags of virtual memory regions safely.
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
    /// Map the current virtual memory region to the given physical memory frames.
    /// This moves the cursor to the end of the region.
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

    /// Unmap the current virtual memory region.
    /// This moves the cursor to the end of the region.
    pub fn unmap(&mut self, len: usize) -> Result<(), ZodiacError> {
        let page_size = self.page_size;
        let vaddr = self.virtual_address;

        self.page_table
            .write()
            .unmap_cont(vaddr, page_size.align_up(len))?;

        self.virtual_address += len;

        Ok(())
    }

    /// Changes the flags of the current virtual memory region.
    /// This moves the cursor to the end of the region.
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
    /// Jump to the given virtual address.
    pub fn jump_to(&mut self, virtual_address: VirtualAddress) -> Result<(), ZodiacError> {
        if !self.page_size.is_aligned(virtual_address) {
            return Err(ZodiacError::InvalidArguments);
        }

        self.virtual_address = virtual_address;
        Ok(())
    }
}

/// Safe interface to read data from a virtual memory space.
pub struct VmReader {
    address: VirtualAddress,
    len: usize,
    page_table: Arc<RwLock<dyn GeneralPageTable>>,
}

impl VmReader {
    /// Read data from the virtual memory space into the buffer.
    /// The virtual memory space doesn't necessarily have to be the current one.
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

/// Safe interface to write data into a virtual memory space.
pub struct VmWriter {
    address: VirtualAddress,
    len: usize,
    page_table: Arc<RwLock<dyn GeneralPageTable>>,
}

impl VmWriter {
    /// Write data from the buffer into the virtual memory space.
    /// The virtual memory space doesn't necessarily have to be the current one.
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

/// Safe wrapper to map a binary file.
/// Supported formats:
/// * elf
/// * pe
pub struct BinaryFileMapper<'a> {
    vm_space: &'a VirtualMemorySpace,
}

impl<'a> BinaryFileMapper<'a> {
    /// Simply map the binary file.
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
