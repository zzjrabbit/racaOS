use core::{fmt::Debug, mem::MaybeUninit};

use alloc::sync::Arc;
use spin::{Lazy, RwLock};

use crate::{
    ZodiacError,
    arch::mem::kernel_page_table,
    io::IoMem,
    mem::{
        GeneralPageTable, Page, PageProperty, PageSize, PhysicalMemory, VirtualAddress,
        convert_physical_to_virtual,
    },
};

pub struct VmSpace {
    page_table: Arc<RwLock<dyn GeneralPageTable>>,
}

impl Debug for VmSpace {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("VmSpace").finish()
    }
}

static KERNEL_VM_SPACE: Lazy<Arc<VmSpace>> = Lazy::new(|| {
    Arc::new(VmSpace {
        page_table: kernel_page_table(),
    })
});

impl VmSpace {
    pub fn kernel() -> Arc<VmSpace> {
        KERNEL_VM_SPACE.clone()
    }

    /// Create a new user virtual memory space.
    /// This is a copy of kernel virtual memory space.
    pub fn new_user() -> Self {
        Self {
            page_table: Self::kernel().page_table.read().deep_copy(),
        }
    }
}

impl VmSpace {
    /// Create a cursor at the given virtual address with the given page size.
    /// So that you can map, unmap and change the flags of virtual memory regions.
    pub fn cursor(&self, virtual_address: VirtualAddress) -> Result<Cursor, ZodiacError> {
        Cursor::new(self.page_table.clone(), virtual_address)
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
}

impl VmSpace {
    // Activate this virtual memory space.
    // Might cause Page Fault if you are not carefull.
    pub fn activate(&self) {
        self.page_table.read().activate();
    }
}

/// An interface to map, unmap and change flags of virtual memory regions safely.
pub struct Cursor {
    page_table: Arc<RwLock<dyn GeneralPageTable>>,
    virtual_address: VirtualAddress,
}

impl Cursor {
    fn new(
        page_table: Arc<RwLock<dyn GeneralPageTable>>,
        virtual_address: VirtualAddress,
    ) -> Result<Self, ZodiacError> {
        if !PageSize::Size4K.is_aligned(virtual_address) {
            return Err(ZodiacError::InvalidArguments);
        }
        Ok(Self {
            page_table,
            virtual_address,
        })
    }
}

impl Cursor {
    /// Map the current virtual memory region to the given physical memory frames.
    /// This moves the cursor to the end of the region.
    pub fn map(
        &mut self,
        physical_memory: &PhysicalMemory,
        property: PageProperty,
    ) -> Result<(), ZodiacError> {
        let page_size = PageSize::Size4K;

        let vaddr = self.virtual_address;
        let page_count = physical_memory.count();
        let mut first_error = None;

        for index in 0..page_count {
            if let Err(error) = self.page_table.write().map(
                Page::new_aligned(vaddr + page_size as usize * index, page_size),
                physical_memory.get_start_address_of_frame(index)?,
                property,
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

    pub fn map_iomem(
        &mut self,
        io_mem: &IoMem,
        property: PageProperty,
        offset: usize,
        len: usize,
    ) -> Result<(), ZodiacError> {
        let page_size = PageSize::Size4K;

        let vaddr = self.virtual_address;
        let phys_start = page_size.align_down(io_mem.start_address() + offset);
        let size = {
            let end = io_mem.start_address() + offset + len;
            end - phys_start
        };
        let page_count = page_size.align_up(size) / page_size as usize;
        let mut first_error = None;

        for index in 0..page_count {
            if let Err(error) = self.page_table.write().map(
                Page::new_aligned(vaddr + page_size as usize * index, page_size),
                phys_start + index * page_size as usize,
                property,
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
        let page_size = PageSize::Size4K;
        let vaddr = self.virtual_address;

        self.page_table
            .write()
            .unmap_cont(vaddr, page_size.align_up(len))?;

        self.virtual_address += len;

        Ok(())
    }

    /// Changes the flags of the current virtual memory region.
    /// This moves the cursor to the end of the region.
    pub fn protect(
        &mut self,
        len: usize,
        updater: impl Fn(&mut PageProperty),
    ) -> Result<(), ZodiacError> {
        let page_size = PageSize::Size4K;
        let vaddr = self.virtual_address;

        let len = page_size.align_up(len);
        let page_count = len / page_size as usize;

        for index in 0..page_count {
            let page_addr = vaddr + page_size as usize * index;

            let (_, mut property, _) = self.page_table.write().query(page_addr)?;
            updater(&mut property);

            self.page_table.write().update(page_addr, property)?;
        }

        self.virtual_address += len;

        Ok(())
    }

    pub fn query(&mut self) -> Result<(PhysicalMemory, PageProperty), ZodiacError> {
        let result = self
            .page_table
            .write()
            .query(self.virtual_address)
            .map(|(paddr, property, _)| (PhysicalMemory::containing_address(paddr, 1), property));
        self.virtual_address += PageSize::Size4K as usize;
        result
    }
}

impl Cursor {
    /// Jump to the given virtual address.
    pub fn jump_to(&mut self, virtual_address: VirtualAddress) -> Result<(), ZodiacError> {
        if !PageSize::Size4K.is_aligned(virtual_address) {
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
    pub fn read_bytes(&self, buffer: &mut [u8]) -> Result<(), ZodiacError> {
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

    /// Read data from the virtual memory space into the buffer.
    /// The virtual memory space doesn't necessarily have to be the current one.
    pub fn read<T: Pod>(&self, buffer: &mut T) -> Result<(), ZodiacError> {
        let buffer = buffer.as_bytes_mut();

        self.read_bytes(buffer)
    }
}

/// Safe interface to write data into a virtual memory space.
pub struct VmWriter {
    address: VirtualAddress,
    len: usize,
    page_table: Arc<RwLock<dyn GeneralPageTable>>,
}

impl VmWriter {
    pub fn write_bytes(&self, buffer: &[u8]) -> Result<(), ZodiacError> {
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

    /// Write data from the buffer into the virtual memory space.
    /// The virtual memory space doesn't necessarily have to be the current one.
    pub fn write<T: Pod>(&self, buffer: &T) -> Result<(), ZodiacError> {
        let buffer = buffer.as_bytes();

        self.write_bytes(buffer)
    }
}

pub unsafe trait Pod: Copy + Sized {
    /// Creates a new instance of Pod type that is filled with zeroes.
    fn new_zeroed() -> Self {
        // SAFETY. An all-zero value of `T: Pod` is always valid.
        unsafe { core::mem::zeroed() }
    }

    /// Creates a new instance of Pod type with uninitialized content.
    fn new_uninit() -> Self {
        // SAFETY. A value of `T: Pod` can have arbitrary bits.
        #[allow(clippy::uninit_assumed_init)]
        unsafe {
            MaybeUninit::uninit().assume_init()
        }
    }

    /// Creates a new instance from the given bytes.
    fn from_bytes(bytes: &[u8]) -> Self {
        let mut new_self = Self::new_uninit();
        let copy_len = new_self.as_bytes().len();
        new_self.as_bytes_mut().copy_from_slice(&bytes[..copy_len]);
        new_self
    }

    /// As a slice of bytes.
    fn as_bytes(&self) -> &[u8] {
        let ptr = self as *const Self as *const u8;
        let len = core::mem::size_of::<Self>();
        unsafe { core::slice::from_raw_parts(ptr, len) }
    }

    /// As a mutable slice of bytes.
    fn as_bytes_mut(&mut self) -> &mut [u8] {
        let ptr = self as *mut Self as *mut u8;
        let len = core::mem::size_of::<Self>();
        unsafe { core::slice::from_raw_parts_mut(ptr, len) }
    }
}

macro_rules! impl_pod_for {
    ($($pod_ty:ty),*) => {
        $(unsafe impl Pod for $pod_ty {})*
    };
}
// impl Pod for primitive types
impl_pod_for!(
    u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, isize, usize
);
// impl Pod for array
unsafe impl<T: Pod, const N: usize> Pod for [T; N] {}
