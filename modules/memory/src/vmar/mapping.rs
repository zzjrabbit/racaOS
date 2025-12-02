use errors::Result;
use mostd::mem::{MMUFlags, PageProperty, VirtualAddress};

use crate::{PAGE_SIZE, Vmo};

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

    pub fn make_not_overlap_with(&mut self, other: &Self) -> (Option<Self>, bool) {
        if self.start() == other.start() && self.size() == other.size() {
            return (None, true);
        }

        if self.contains_range(other.start(), other.size()) {
            let new_self_size = self.end() - other.end();
            let new_self = Self::new(
                self.vmo
                    .split((self.vmo.len() - new_self_size) / PAGE_SIZE)
                    .unwrap(),
                other.end(),
                new_self_size,
                self.prop,
                self.perm,
            );
            self.size = other.start() - self.start();
            (Some(new_self), false)
        } else if other.contains_range(self.start(), self.size()) {
            (None, true)
        } else if self.start() <= other.end() {
            self.start = other.end();
            (None, false)
        } else if other.start() <= self.end() {
            self.size = other.start() - self.start();
            (None, false)
        } else {
            (None, false)
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
