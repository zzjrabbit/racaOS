use alloc::sync::Arc;
use bitflags::bitflags;
use spin::RwLock;

use crate::{
    UnmapError, ZodiacError,
    mem::{PhysicalAddress, VirtualAddress},
};

/// Page Size
/// Interfaces to support huge page.
#[repr(usize)]
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub enum PageSize {
    #[default]
    Size4K = 0x1000,
    Size2M = 0x20_0000,
    Size1G = 0x4000_0000,
}

impl TryFrom<usize> for PageSize {
    type Error = ();
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0x1000 => Ok(Self::Size4K),
            0x20_0000 => Ok(Self::Size2M),
            0x4000_0000 => Ok(Self::Size1G),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Page {
    pub vaddr: VirtualAddress,
    pub size: PageSize,
}

impl PageSize {
    pub const fn is_aligned(self, addr: usize) -> bool {
        self.page_offset(addr) == 0
    }

    pub const fn align_down(self, addr: usize) -> usize {
        addr & !(self as usize - 1)
    }

    pub const fn align_up(self, addr: usize) -> usize {
        self.align_down(addr + self as usize - 1)
    }

    pub const fn page_offset(self, addr: usize) -> usize {
        addr & (self as usize - 1)
    }

    pub const fn is_huge(self) -> bool {
        matches!(self, Self::Size1G | Self::Size2M)
    }
}

impl Page {
    pub fn new_aligned(vaddr: VirtualAddress, size: PageSize) -> Self {
        Self { vaddr, size }
    }
}

bitflags! {
    /// Flags for mapping.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct MMUFlags: u32 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const USER = 1 << 2;
        const EXECUTE = 1 << 3;
        const HUGE_PAGE = 1 << 4;

        const KERNEL_DATA = Self::READ.bits() | Self::WRITE.bits();
        const KERNEL_CODE = Self::READ.bits() | Self::EXECUTE.bits();

        const USER_DATA = Self::KERNEL_DATA.bits() | Self::USER.bits();
        const USER_CODE = Self::KERNEL_CODE.bits() | Self::USER.bits();
    }
}

#[allow(dead_code)]
pub trait GeneralPageTable: Sync + Send {
    fn physical_address(&self) -> PhysicalAddress;
    fn map(
        &mut self,
        page: Page,
        paddr: PhysicalAddress,
        flags: MMUFlags,
    ) -> Result<(), ZodiacError>;
    fn unmap(&mut self, vaddr: VirtualAddress) -> Result<(PhysicalAddress, PageSize), ZodiacError>;
    fn update(&mut self, vaddr: VirtualAddress, flags: MMUFlags) -> Result<PageSize, ZodiacError>;

    /// Note that the returned physical address is not necessarily aligned by page size.
    /// Simple Case: Page 0 is mapped to Frame 0, then querring 0x10 will get physical address 0x10.
    fn query(
        &mut self,
        vaddr: VirtualAddress,
    ) -> Result<(PhysicalAddress, MMUFlags, PageSize), ZodiacError>;
    fn deep_copy(&self, remove_write: bool) -> Arc<RwLock<dyn GeneralPageTable>>;
    fn switch(&self);

    /// Note that start_vaddr and size must be aligned by page size
    fn map_cont(
        &mut self,
        start_vaddr: VirtualAddress,
        size: usize,
        start_paddr: PhysicalAddress,
        flags: MMUFlags,
    ) -> Result<(), ZodiacError> {
        let mut vaddr = start_vaddr;
        let mut paddr = start_paddr;
        let end_vaddr = vaddr + size;
        if flags.contains(MMUFlags::HUGE_PAGE) {
            while vaddr < end_vaddr {
                let remains = end_vaddr - vaddr;
                let page_size = if remains >= PageSize::Size1G as usize
                    && PageSize::Size1G.is_aligned(vaddr)
                    && PageSize::Size1G.is_aligned(paddr)
                {
                    PageSize::Size1G
                } else if remains >= PageSize::Size2M as usize
                    && PageSize::Size2M.is_aligned(vaddr)
                    && PageSize::Size2M.is_aligned(paddr)
                {
                    PageSize::Size2M
                } else {
                    PageSize::Size4K
                };
                let page = Page::new_aligned(vaddr, page_size);
                self.map(page, paddr, flags)?;
                vaddr += page_size as usize;
                paddr += page_size as usize;
            }
        } else {
            while vaddr < end_vaddr {
                let page_size = PageSize::Size4K;
                let page = Page::new_aligned(vaddr, page_size);
                self.map(page, paddr, flags)?;
                vaddr += page_size as usize;
                paddr += page_size as usize;
            }
        }
        Ok(())
    }

    /// Note that start_vaddr and size must be aligned by page size.
    fn unmap_cont(&mut self, start_vaddr: VirtualAddress, size: usize) -> Result<(), ZodiacError> {
        let mut vaddr = start_vaddr;
        let end_vaddr = vaddr + size;
        while vaddr < end_vaddr {
            let page_size = match self.unmap(vaddr) {
                Ok((_, s)) => {
                    assert!(s.is_aligned(vaddr));
                    s as usize
                }
                Err(ZodiacError::FailedToUnmap(UnmapError::NotMappedYet)) => {
                    PageSize::Size4K as usize
                }
                Err(e) => return Err(e),
            };
            vaddr += page_size;
            assert!(vaddr <= end_vaddr);
        }
        Ok(())
    }
}
