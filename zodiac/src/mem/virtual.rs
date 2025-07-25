use alloc::{sync::Arc, vec::Vec};
use limine::request::{ExecutableAddressRequest, ExecutableFileRequest};
use spin::{Lazy, Mutex, RwLock};

use crate::{
    MapError, UnmapError, UpdateError, ZodiacError,
    hal::mem::*,
    mem::{GeneralPageTable, MMUFlags, Page, PhysicalMemory, VirtualAddress},
};

pub struct VirtualMemory {
    start_address: VirtualAddress,
    len: usize,
    inner: RwLock<VirtualMemoryInner>,
    page_table: Arc<RwLock<dyn GeneralPageTable>>,
    father: Option<Arc<VirtualMemory>>,
    allocation_lock: Mutex<()>,
}

struct VirtualMemoryInner {
    children: Vec<Arc<VirtualMemory>>,
    mappings: Vec<Arc<VmMapping>>,
}

#[used]
#[unsafe(link_section = ".requests")]
static EXECUTABLE_FILE_REQUEST: ExecutableFileRequest = ExecutableFileRequest::new();

#[used]
#[unsafe(link_section = ".requests")]
static EXECUTABLE_ADDRESS_REQUEST: ExecutableAddressRequest = ExecutableAddressRequest::new();

static KERNEL_VM: Lazy<Arc<VirtualMemory>> = Lazy::new(|| {
    let vm = VirtualMemory::new_kernel();

    let kernel_size = EXECUTABLE_FILE_REQUEST
        .get_response()
        .unwrap()
        .file()
        .size() as usize;
    let kernel_address = EXECUTABLE_ADDRESS_REQUEST
        .get_response()
        .unwrap()
        .virtual_base() as VirtualAddress;

    let _kernel_vm = vm.allocate(Some(kernel_address - vm.start_address), kernel_size, 1);

    vm
});

impl VirtualMemory {
    pub(crate) fn new_kernel() -> Arc<Self> {
        Arc::new(Self {
            start_address: KERNEL_ASPACE_BASE,
            len: KERNEL_ASPACE_SIZE,
            inner: RwLock::new(VirtualMemoryInner {
                children: Vec::new(),
                mappings: Vec::new(),
            }),
            page_table: kernel_page_table(),
            father: None,
            allocation_lock: Mutex::new(()),
        })
    }

    pub fn new_user() -> Arc<Self> {
        Arc::new(Self {
            start_address: USER_ASPACE_BASE,
            len: USER_ASPACE_SIZE,
            inner: RwLock::new(VirtualMemoryInner {
                children: Vec::new(),
                mappings: Vec::new(),
            }),
            page_table: kernel_page_table().read().deep_copy(),
            father: None,
            allocation_lock: Mutex::new(()),
        })
    }

    pub fn kernel() -> Arc<Self> {
        KERNEL_VM.clone()
    }
}

impl VirtualMemory {
    pub fn father(&self) -> Option<Arc<Self>> {
        self.father.clone()
    }
}

impl VirtualMemory {
    pub fn allocate(
        self: &Arc<Self>,
        offset: Option<usize>,
        len: usize,
        align: usize,
    ) -> Result<Arc<Self>, ZodiacError> {
        let _allocation_guard = self.allocation_lock.lock();
        
        let offset = self.determine_offset(offset, len, align)?;
        let child = Arc::new(Self {
            start_address: self.start_address + offset,
            len,
            inner: RwLock::new(VirtualMemoryInner {
                children: Vec::new(),
                mappings: Vec::new(),
            }),
            page_table: self.page_table.clone(),
            father: Some(self.clone()),
            allocation_lock: Mutex::new(()),
        });
        self.inner.write().children.push(child.clone());
        Ok(child)
    }

    fn determine_offset(
        &self,
        offset: Option<usize>,
        len: usize,
        align: usize,
    ) -> Result<usize, ZodiacError> {
        if len % align != 0 {
            Err(ZodiacError::InvalidArguments)
        } else if let Some(offset) = offset {
            if (offset + self.start_address) % align == 0 && self.test_map(offset, len, align) {
                Ok(offset)
            } else {
                Err(ZodiacError::InvalidArguments)
            }
        } else if len > self.len {
            Err(ZodiacError::InvalidArguments)
        } else {
            match self.find_free_area(0, len, align) {
                Some(offset) => Ok(offset),
                None => Err(ZodiacError::NoMemory),
            }
        }
    }

    fn test_map(&self, offset: usize, len: usize, align: usize) -> bool {
        if (offset + self.start_address) % align != 0 || len % align != 0 {
            return false;
        }

        let start = self.start_address + offset;
        let end = start + len;
        if end > self.start_address + self.len {
            return false;
        }
        
        let inner = self.inner.read();
        
        if inner.children
            .iter()
            .any(|vm| vm.overlap(start, end))
        {
            return false;
        }
        if inner.mappings
            .iter()
            .any(|map| map.overlap(start, end))
        {
            return false;
        }
        true
    }

    fn find_free_area(&self, offset_hint: usize, len: usize, align: usize) -> Option<usize> {
        let inner = self.inner.read();
        core::iter::once(offset_hint)
            .chain(
                inner
                    .children
                    .iter()
                    .map(|child| child.end_address() - self.start_address),
            )
            .chain(
                inner
                    .mappings
                    .iter()
                    .map(|mapping| mapping.end_address() - self.start_address),
            )
            .find(|&offset| self.test_map(offset, len, align))
    }
}

impl VirtualMemory {
    pub fn map(
        &self,
        offset: usize,
        physical_memory: Arc<PhysicalMemory>,
        flags: MMUFlags,
    ) -> Result<(), MapError> {
        if (self.start_address + offset) % physical_memory.page_size() as usize != 0 {
            return Err(MapError::VirtualAddressNotAligned);
        }

        if !self.test_map(
            offset,
            physical_memory.count() * physical_memory.page_size() as usize,
            physical_memory.page_size() as usize,
        ) {
            return Err(MapError::PageAlreadyMapped.into());
        }

        let vm_mapping = VmMapping::new(
            self.start_address + offset,
            physical_memory.count() * physical_memory.page_size() as usize,
            physical_memory.clone(),
            flags,
            self.page_table.clone(),
        );
        self.inner.write().mappings.push(vm_mapping);

        Ok(())
    }

    pub fn unmap(&self, offset: usize, len: usize) -> Result<(), ZodiacError> {
        if offset + len > self.len {
            return Err(ZodiacError::OutOfBounds);
        }

        let start = self.start_address + offset;
        let end = start + len;

        if let Some(mapping) = self
            .inner
            .read()
            .mappings
            .iter()
            .find(|map| map.overlap(start, end))
        {
            mapping.unmap(self.start_address + offset, len)?;
        } else if let Some(child) = self
            .inner
            .read()
            .children
            .iter()
            .find(|ch| ch.overlap(start, end))
        {
            return child.unmap(offset, len);
        } else {
            return Err(UnmapError::NotMappedYet.into());
        }

        self.inner
            .write()
            .mappings
            .retain(|map| !map.overlap(start, end));

        Ok(())
    }

    pub fn protect(&self, offset: usize, len: usize, flags: MMUFlags) -> Result<(), ZodiacError> {
        if offset + len > self.len {
            return Err(ZodiacError::OutOfBounds);
        }

        let start = self.start_address + offset;
        let end = start + len;

        if let Some(mapping) = self
            .inner
            .read()
            .mappings
            .iter()
            .find(|map| map.overlap(start, end))
        {
            mapping.protect(self.start_address + offset, len, flags)?;
        } else if let Some(child) = self
            .inner
            .read()
            .children
            .iter()
            .find(|ch| ch.overlap(start, end))
        {
            return child.protect(offset, len, flags);
        } else {
            return Err(UpdateError::NotMappedYet.into());
        }

        self.inner
            .write()
            .mappings
            .retain(|map| !map.overlap(start, end));

        Ok(())
    }

    pub fn handle_page_fault(&self, vaddr: VirtualAddress) -> Result<(), ZodiacError> {
        if !self.contains(vaddr) {
            return Err(ZodiacError::NotFound);
        }

        let inner = self.inner.read();
        if let Some(child) = inner.children.iter().find(|ch| ch.contains(vaddr)) {
            return child.handle_page_fault(vaddr);
        }
        if let Some(mapping) = inner.mappings.iter().find(|map| map.contains(vaddr)) {
            return mapping.handle_page_fault(vaddr);
        }
        Err(ZodiacError::NotFound)
    }
}

impl VirtualMemory {
    pub fn end_address(&self) -> VirtualAddress {
        self.start_address + self.len
    }

    pub fn overlap(&self, start: VirtualAddress, end: VirtualAddress) -> bool {
        !(self.start_address >= end || self.end_address() <= start)
    }

    pub fn contains(&self, vaddr: VirtualAddress) -> bool {
        self.start_address <= vaddr && vaddr < self.end_address()
    }

    pub fn start_address(&self) -> VirtualAddress {
        self.start_address
    }
}

struct VmMapping {
    start_address: VirtualAddress,
    size: usize,
    physical_memory: Arc<PhysicalMemory>,
    page_table: Arc<RwLock<dyn GeneralPageTable>>,
    inner: RwLock<VmMappingInner>,
}

struct VmMappingInner {
    flags: MMUFlags,
}

impl VmMapping {
    fn new(
        start_address: VirtualAddress,
        size: usize,
        physical_memory: Arc<PhysicalMemory>,
        flags: MMUFlags,
        page_table: Arc<RwLock<dyn GeneralPageTable>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            start_address,
            size,
            physical_memory,
            page_table,
            inner: RwLock::new(VmMappingInner { flags }),
        })
    }
}

impl VmMapping {
    fn map(self: &Arc<Self>, vaddr: VirtualAddress, size: usize) -> Result<(), ZodiacError> {
        if vaddr < self.start_address || vaddr + size > self.start_address + self.size {
            return Err(ZodiacError::OutOfBounds);
        }

        let page_size = self.physical_memory.page_size();

        let vaddr = page_size.align_down(vaddr);
        let size = page_size.align_up(size);
        let page_count = size / page_size as usize;
        let start_id = (vaddr - self.start_address) / page_size as usize;

        if self.physical_memory.contiguous() {
            self.page_table.write().map_cont(
                self.start_address,
                self.size,
                self.physical_memory.get_start_address_of_frame(0)?,
                self.inner.read().flags,
            )?;
        } else {
            for index in 0..page_count {
                self.page_table.write().map(
                    Page::new_aligned(vaddr + page_size as usize * index, page_size),
                    self.physical_memory
                        .get_start_address_of_frame(start_id + index)?,
                    self.inner.read().flags,
                )?;
            }
        }

        Ok(())
    }

    fn unmap(self: &Arc<Self>, vaddr: VirtualAddress, size: usize) -> Result<(), ZodiacError> {
        if vaddr < self.start_address || vaddr + size > self.start_address + self.size {
            return Err(ZodiacError::OutOfBounds);
        }

        let page_size = self.physical_memory.page_size();

        let vaddr = page_size.align_down(vaddr);
        let size = page_size.align_up(size);
        let page_count = size / page_size as usize;

        for index in 0..page_count {
            self.page_table
                .write()
                .unmap(vaddr + page_size as usize * index)?;
        }

        Ok(())
    }

    fn protect(
        &self,
        vaddr: VirtualAddress,
        size: usize,
        flags: MMUFlags,
    ) -> Result<(), ZodiacError> {
        if vaddr < self.start_address || vaddr + size > self.start_address + self.size {
            return Err(ZodiacError::OutOfBounds);
        }

        let page_size = self.physical_memory.page_size();

        let vaddr = page_size.align_down(vaddr);
        let size = page_size.align_up(size);
        let page_count = size / page_size as usize;

        for index in 0..page_count {
            self.page_table
                .write()
                .update(vaddr + page_size as usize * index, flags)?;
        }

        Ok(())
    }
}

impl VmMapping {
    fn handle_page_fault(self: &Arc<Self>, vaddr: VirtualAddress) -> Result<(), ZodiacError> {
        self.map(vaddr, self.physical_memory.page_size() as usize)
    }
}

impl VmMapping {
    fn end_address(&self) -> VirtualAddress {
        self.start_address + self.size
    }

    fn overlap(&self, start: VirtualAddress, end: VirtualAddress) -> bool {
        !(self.start_address >= end || self.end_address() <= start)
    }

    fn contains(&self, vaddr: VirtualAddress) -> bool {
        self.start_address <= vaddr && vaddr < self.end_address()
    }
}
