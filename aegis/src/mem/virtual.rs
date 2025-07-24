use alloc::{sync::Arc, vec::Vec};
use spin::{Lazy, RwLock};

use crate::{
    hal::mem::*, mem::{GeneralPageTable, MMUFlags, Page, PhysicalMemory, VirtualAddress}, AegisError, MapError, UnmapError, UpdateError
};

pub struct VirtualMemory {
    start_address: VirtualAddress,
    len: usize,
    inner: RwLock<VirtualMemoryInner>,
    page_table: Arc<RwLock<dyn GeneralPageTable>>,
    father: Option<Arc<VirtualMemory>>,
}

struct VirtualMemoryInner {
    children: Vec<Arc<VirtualMemory>>,
    mappings: Vec<Arc<VmMapping>>,
}

static KERNEL_VM: Lazy<Arc<VirtualMemory>> = Lazy::new(VirtualMemory::new_kernel);

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
    ) -> Result<Arc<Self>, AegisError> {
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
        });
        self.inner.write().children.push(child.clone());
        Ok(child)
    }

    fn determine_offset(
        &self,
        offset: Option<usize>,
        len: usize,
        align: usize,
    ) -> Result<usize, AegisError> {
        if len % align != 0 {
            Err(AegisError::InvalidArguments)
        } else if let Some(offset) = offset {
            if (offset + self.start_address) % align == 0 && self.test_map(offset, len, align) {
                Ok(offset)
            } else {
                Err(AegisError::InvalidArguments)
            }
        } else if len > self.len {
            Err(AegisError::InvalidArguments)
        } else {
            match self.find_free_area(0, len, align) {
                Some(offset) => Ok(offset),
                None => Err(AegisError::NoMemory),
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
        if self
            .inner
            .read()
            .children
            .iter()
            .any(|vm| vm.overlap(start, end))
        {
            return false;
        }
        if self
            .inner
            .read()
            .mappings
            .iter()
            .any(|map| map.overlap(start, end))
        {
            return false;
        }
        true
    }

    fn find_free_area(&self, offset_hint: usize, len: usize, align: usize) -> Option<usize> {
        core::iter::once(offset_hint)
            .chain(
                self.inner
                    .read()
                    .children
                    .iter()
                    .map(|child| child.end_address() - self.start_address),
            )
            .chain(
                self.inner
                    .read()
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
        
        if !self.test_map(offset, physical_memory.count() * physical_memory.page_size() as usize, physical_memory.page_size() as usize) {
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
    
    pub fn unmap(&self, offset: usize, len: usize) -> Result<(), AegisError> {
        if offset + len > self.len {
            return Err(AegisError::OutOfBounds);
        }
        
        let start = self.start_address + offset;
        let end = start + len;
        
        if let Some(mapping) = self.inner.read().mappings.iter().find(|map| map.overlap(start, end)) {
            mapping.unmap()?;
        } else if let Some(child) = self.inner.read().children.iter().find(|ch| ch.overlap(start, end)) {
            return child.unmap(offset, len);
        } else {
            return Err(UnmapError::NotMappedYet.into());
        }
        
        self.inner.write().mappings.retain(|map| !map.overlap(start, end));
        
        Ok(())
    }
    
    pub fn protect(&self, offset: usize, len: usize, flags: MMUFlags) -> Result<(), AegisError> {
        if offset + len > self.len {
            return Err(AegisError::OutOfBounds);
        }
        
        let start = self.start_address + offset;
        let end = start + len;
        
        if let Some(mapping) = self.inner.read().mappings.iter().find(|map| map.overlap(start, end)) {
            mapping.protect(flags)?;
        } else if let Some(child) = self.inner.read().children.iter().find(|ch| ch.overlap(start, end)) {
            return child.protect(offset, len, flags);
        } else {
            return Err(UpdateError::NotMappedYet.into());
        }
        
        self.inner.write().mappings.retain(|map| !map.overlap(start, end));
        
        Ok(())
    }

    pub fn handle_page_fault(
        &self,
        vaddr: VirtualAddress,
        flags: MMUFlags,
    ) -> Result<(), AegisError> {
        if !self.contains(vaddr) {
            return Err(AegisError::NotFound);
        }

        let inner = self.inner.read();
        if let Some(child) = inner.children.iter().find(|ch| ch.contains(vaddr)) {
            return child.handle_page_fault(vaddr, flags);
        }
        if let Some(mapping) = inner.mappings.iter().find(|map| map.contains(vaddr)) {
            return mapping.handle_page_fault(vaddr, flags);
        }
        Err(AegisError::NotFound)
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
    mapped: bool,
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
            inner: RwLock::new(VmMappingInner { flags, mapped: false }),
        })
    }
}

impl VmMapping {
    fn mapped(&self) -> bool {
        self.inner.read().mapped
    }
    
    fn map(self: &Arc<Self>) -> Result<(), AegisError> {
        if self.mapped() {
            return Ok(());
        }
        
        if self.physical_memory.contiguous() {
            self.page_table.write().map_cont(
                self.start_address,
                self.size,
                self.physical_memory.get_start_address_of_frame(0)?,
                self.inner.read().flags,
            )?;
            self.inner.write().mapped = true;
        } else {
            let page_size = self.physical_memory.page_size();
            for index in 0..self.physical_memory.count() {
                self.page_table.write().map(
                    Page::new_aligned(self.start_address + page_size as usize * index, page_size),
                    self.physical_memory.get_start_address_of_frame(index)?,
                    self.inner.read().flags,
                )?;
            }
            self.inner.write().mapped = true;
        }
        
        Ok(())
    }
    
    fn unmap(self: &Arc<Self>) -> Result<(), AegisError> {
        if !self.mapped() {
            return Ok(());
        }
        
        if self.physical_memory.contiguous() {
            self.page_table.write().unmap_cont(
                self.start_address,
                self.size,
            )?;
        } else {
            let page_size = self.physical_memory.page_size();
            for index in 0..self.physical_memory.count() {
                self.page_table.write().unmap(
                    self.start_address + page_size as usize * index,
                )?;
            }
        }
        self.inner.write().mapped = false;
        Ok(())
    }
    
    fn protect(&self, flags: MMUFlags) -> Result<(), AegisError> {
        for index in 0..self.physical_memory.count() {
            let address = self.start_address + index * self.physical_memory.page_size() as usize;
            self.page_table.write().update(address, flags)?;
        }
        Ok(())
    }
}

impl VmMapping {
    fn handle_page_fault(self: &Arc<Self>, _vaddr: VirtualAddress, _flags: MMUFlags) -> Result<(), AegisError> {
        self.map()
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
