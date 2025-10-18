use errors::Result;
use ostd::mm::{PageFlags, PageProperty, Vaddr};

use crate::Vmo;

#[derive(Debug)]
pub struct VmMapping {
    vmo: Vmo,
    start: Vaddr,
    size: usize,
    prop: PageProperty,
    perm: PageFlags,
}

impl VmMapping {
    pub fn new(vmo: Vmo, start: Vaddr, size: usize, prop: PageProperty, perm: PageFlags) -> Self {
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

    pub fn start(&self) -> Vaddr {
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

    pub fn perm(&self) -> PageFlags {
        self.perm
    }

    pub fn set_perm(&mut self, perm: PageFlags) {
        self.perm = perm;
    }

    pub fn overlaps(&self, other: &VmMapping) -> bool {
        self.start <= other.start + other.size && other.start < self.start + self.size
    }

    pub fn contains_range(&self, start: Vaddr, size: usize) -> bool {
        self.start <= start && start + size <= self.start + self.size
    }

    pub fn contains(&self, addr: Vaddr) -> bool {
        self.start <= addr && addr < self.start + self.size
    }
}

impl VmMapping {
    pub fn clone(&mut self) -> Result<Self> {
        let mut prop = self.prop;
        prop.flags.remove(PageFlags::W);

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
