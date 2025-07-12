use crate::{RcResult, syscall};

pub struct VirtualMemory(u32);
pub struct PhysicalMemory(u32);

impl VirtualMemory {
    pub fn allocate_child(&self, page_count: usize) -> RcResult<Self> {
        let mut handle = 0u32;
        syscall!(2, self.0, &raw mut handle, page_count)?;
        Ok(Self(handle))
    }

    pub fn start_address(&self) -> RcResult<usize> {
        syscall!(3, self.0)
    }

    pub fn create_child(&self, start_page: usize, page_count: usize) -> RcResult<Self> {
        let mut child_handle = 0u32;
        syscall!(8, self.0, start_page, page_count, &raw mut child_handle)?;
        Ok(Self(child_handle))
    }

    pub fn root_virtual_memory() -> RcResult<Self> {
        let mut handle = 0u32;
        syscall!(9, &raw mut handle)?;
        Ok(Self(handle))
    }

    pub fn as_handle(&self) -> u32 {
        self.0
    }
}

impl PhysicalMemory {
    pub fn create(count: usize) -> RcResult<Self> {
        let mut handle = 0u32;
        syscall!(4, count, &raw mut handle)?;
        Ok(Self(handle))
    }

    pub fn start_address(&self) -> RcResult<usize> {
        syscall!(5, self.0)
    }
}

bitflags::bitflags! {
    /// Generic memory flags.
    #[derive(Clone, Copy, Debug)]
    pub struct MMUFlags: usize {
        const READ      = 1 << 2;
        const WRITE     = 1 << 3;
        const EXECUTE   = 1 << 4;
        const RXW = Self::READ.bits() | Self::WRITE.bits() | Self::EXECUTE.bits();
    }
}

impl VirtualMemory {
    pub fn map(&self, pm: PhysicalMemory, flags: MMUFlags) -> RcResult<()> {
        syscall!(6, self.0, pm.0, flags.bits()).map(|_| {})
    }

    pub fn unmap(&self) -> RcResult<()> {
        syscall!(7, self.0).map(|_| {})
    }
}
