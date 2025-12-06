use errors::{Errno, Result};
use mostd::mem::{MMUFlags, PageProperty, VirtualAddress};

use crate::{PAGE_SIZE, Vmo, align_down_by_page_size};

#[derive(Debug)]
pub struct VmMapping {
    vmo: Vmo,
    start: VirtualAddress,
    size: usize,
    prop: PageProperty,
    perm: MMUFlags,
}

impl VmMapping {
    pub fn new(
        vmo: Vmo,
        start: VirtualAddress,
        size: usize,
        prop: PageProperty,
        perm: MMUFlags,
    ) -> Self {
        VmMapping {
            vmo,
            start,
            size,
            prop,
            perm,
        }
    }
}

#[allow(dead_code)]
impl VmMapping {
    pub fn vmo(&self) -> &Vmo {
        &self.vmo
    }

    pub fn vmo_mut(&mut self) -> &mut Vmo {
        &mut self.vmo
    }

    pub fn start(&self) -> VirtualAddress {
        self.start
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn prop(&self) -> PageProperty {
        self.prop
    }

    pub fn set_prop(&mut self, prop: PageProperty) {
        self.prop = prop;
    }

    pub fn perm(&self) -> MMUFlags {
        self.perm
    }

    pub fn set_perm(&mut self, perm: MMUFlags) {
        self.perm = perm;
    }

    pub fn overlaps(&self, other: &VmMapping) -> bool {
        self.start < other.end() && other.start < self.end()
    }

    pub fn contains_range(&self, start: VirtualAddress, size: usize) -> bool {
        self.start <= start && start + size <= self.end()
    }

    pub fn contains(&self, addr: VirtualAddress) -> bool {
        self.start <= addr && addr < self.end()
    }

    pub fn end(&self) -> VirtualAddress {
        self.start + self.size
    }

    pub fn split_at(self, addr: VirtualAddress) -> Result<(VmMapping, VmMapping)> {
        if !self.contains(addr) || addr % PAGE_SIZE != 0 {
            return Err(Errno::EINVAL.no_message());
        }

        let offset = addr - self.start();

        let left_vmo = self.vmo().clone();
        let right_vmo = self
            .vmo()
            .split(align_down_by_page_size(offset) / PAGE_SIZE)?;

        let left = Self::new(left_vmo, self.start(), offset, self.prop(), self.perm());
        let right = Self::new(
            right_vmo,
            addr,
            self.size() - offset,
            self.prop(),
            self.perm(),
        );

        Ok((left, right))
    }

    pub fn split_range(
        self,
        left: VirtualAddress,
        right: VirtualAddress,
    ) -> Result<(Option<Self>, Self, Option<Self>)> {
        if left % PAGE_SIZE != 0 || right % PAGE_SIZE != 0 {
            return Err(Errno::EINVAL.no_message());
        }

        let start = self.start();
        let end = self.end();

        if left <= start && right >= end {
            Ok((None, self, None))
        } else if start < left {
            let (left, within) = self.split_at(left)?;
            if right < end {
                let (within, right) = within.split_at(right)?;
                Ok((Some(left), within, Some(right)))
            } else {
                Ok((Some(left), within, None))
            }
        } else if right < end {
            let (within, right) = self.split_at(right)?;
            Ok((None, within, Some(right)))
        } else {
            log::warn!(
                "The mapping {:x}..{:x} does not contain range {:x}..{:x}!",
                start,
                end,
                left,
                right
            );
            Err(Errno::EINVAL.no_message())
        }
    }
}

impl VmMapping {
    pub fn clone(&mut self) -> Result<Self> {
        let mut prop = self.prop;
        prop.flags.remove(MMUFlags::WRITE);

        self.set_prop(prop);

        Ok(Self::new(
            self.vmo.clone(),
            self.start,
            self.size,
            prop,
            self.perm,
        ))
    }
}
