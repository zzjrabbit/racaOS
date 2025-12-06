use core::ops::Range;

use alloc::{sync::Arc, vec::Vec};
use errors::{Errno, Result};
use mostd::{
    arch::mem::MAX_USERSPACE_VADDR,
    mem::{MMUFlags, PageProperty, VirtualAddress, VmSpace},
};
use spin::RwLock;

use mapping::VmMapping;

use crate::{PAGE_SIZE, Vmo, align_down_by_page_size, align_up_by_page_size};

mod mapping;
mod pf;
mod rw;

#[derive(Debug)]
pub struct Vmar {
    vm_space: Arc<VmSpace>,
    inner: RwLock<VmarInner>,
}

#[derive(Debug)]
struct VmarInner {
    vm_mappings: Vec<VmMapping>,
}

impl Vmar {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            vm_space: Arc::new(VmSpace::new_user()),
            inner: RwLock::new(VmarInner {
                vm_mappings: Vec::new(),
            }),
        })
    }
}

impl Vmar {
    pub fn activate(&self) {
        self.vm_space.activate();
    }
}

impl Vmar {
    pub fn map(
        &self,
        addr: VirtualAddress,
        size: usize,
        prop: PageProperty,
        process_overlap: bool,
    ) -> Result<Vmo> {
        if size == 0 {
            return Vmo::allocate_ram(0);
        }

        let aligned = align_down_by_page_size(addr);
        let size = align_up_by_page_size(size + addr - aligned);

        let vmo = Vmo::allocate_ram(size / PAGE_SIZE)?;

        let vm_mapping = VmMapping::new(vmo.clone(), aligned, size, prop, prop.flags);

        if process_overlap {
            self.insert_truncate_others(vm_mapping)?;
        } else {
            self.insert(vm_mapping)?;
        }

        Ok(vmo)
    }

    pub fn map_with_alloc(&self, size: usize, prop: PageProperty) -> Result<(VirtualAddress, Vmo)> {
        let regions = self
            .inner
            .read()
            .vm_mappings
            .iter()
            .map(|mapping| (mapping.start(), mapping.end()))
            .collect::<Vec<_>>();

        const USER_ASPACE_BASE: usize = 0x0000_0001_0000_0000;
        let mut last_end = USER_ASPACE_BASE;

        for (start, end) in regions {
            if start < last_end {
                continue;
            }

            let usable = start - last_end;
            if usable >= size {
                let vmo = self.map(last_end, size, prop, true)?;
                return Ok((last_end, vmo));
            }
            last_end = end;
        }

        if last_end >= MAX_USERSPACE_VADDR {
            Err(Errno::ENOMEM.no_message())
        } else {
            let vmo = self.map(last_end, size, prop, true)?;
            Ok((last_end, vmo))
        }
    }

    pub fn unmap(&self, addr: VirtualAddress, size: usize) -> Result<()> {
        if size == 0 {
            return Ok(());
        }

        let mut inner = self.inner.write();

        for mapping in inner.vm_mappings.iter_mut() {
            if mapping.contains_range(addr, size) {
                let aligned = align_down_by_page_size(addr);
                let size = align_up_by_page_size(size + addr - aligned);

                let mut cursor = self.vm_space.cursor(aligned)?;
                cursor.unmap(size)?;
            }
        }

        inner
            .vm_mappings
            .retain(|mapping| !mapping.contains_range(addr, size));

        Ok(())
    }

    pub fn protect(&self, addr: VirtualAddress, size: usize, flags: MMUFlags) -> Result<()> {
        if size == 0 {
            return Ok(());
        }

        {
            let aligned = align_down_by_page_size(addr);
            let size = align_up_by_page_size(size + addr - aligned);

            let mut cursor = self.vm_space.cursor(aligned)?;
            cursor.protect(size, |cprop| {
                cprop.flags.insert(flags);
            })?;
        }

        let mappings = self
            .inner
            .read()
            .vm_mappings
            .iter()
            .filter(|mapping| mapping.contains_range(addr, size))
            .map(|mapping| mapping.start())
            .collect::<Vec<_>>();

        for &vm_mapping_addr in mappings.iter() {
            let mapping = self.remove_by_addr(vm_mapping_addr).unwrap();

            if mapping.perm().contains(flags) {
                continue;
            }

            let split_range =
                get_intersected_range(&(mapping.start()..mapping.end()), &(addr..addr + size));
            let split_left = split_range.start;
            let split_right = split_range.end;

            let (left, mut taken, right) = mapping.split_range(split_left, split_right)?;

            if let Some(left) = left {
                self.insert(left)?;
            }
            if let Some(right) = right {
                self.insert(right)?;
            }

            let mut prop = taken.prop();
            prop.flags |= flags;
            taken.set_prop(prop);

            taken.set_perm(taken.perm() | flags);

            self.insert(taken)?;
        }

        Ok(())
    }
}

impl Vmar {
    fn remove_by_addr(&self, addr: VirtualAddress) -> Option<VmMapping> {
        let index = self
            .inner
            .read()
            .vm_mappings
            .iter()
            .position(|mapping| mapping.start() == addr)?;
        let mapping = self.inner.write().vm_mappings.remove(index);
        Some(mapping)
    }

    fn insert(&self, mapping: VmMapping) -> Result<()> {
        let mut inner = self.inner.write();
        inner.vm_mappings.push(mapping);
        Ok(())
    }

    fn insert_truncate_others(&self, mapping: VmMapping) -> Result<()> {
        let mappings_to_remove = self
            .inner
            .read()
            .vm_mappings
            .iter()
            .filter(|other| other.overlaps(&mapping))
            .map(|mapping| mapping.start())
            .collect::<Vec<_>>();

        for addr in mappings_to_remove {
            let vm_mapping = self.remove_by_addr(addr).unwrap();

            let vm_mapping_range = vm_mapping.start()..vm_mapping.end();
            let required_range = mapping.start()..mapping.end();

            let split_range = get_intersected_range(&vm_mapping_range, &required_range);
            let split_left = split_range.start;
            let split_right = split_range.end;

            let (left, _taken, right) = vm_mapping.split_range(split_left, split_right)?;
            if let Some(left) = left {
                self.insert(left)?;
            }
            if let Some(right) = right {
                self.insert(right)?;
            }
        }

        self.insert(mapping)?;
        Ok(())
    }
}

impl Vmar {
    pub fn deep_clone(&self) -> Result<Arc<Self>> {
        let mut vm_mappings = Vec::new();
        for mapping in self.inner.write().vm_mappings.iter_mut() {
            vm_mappings.push(mapping.clone()?);
            if mapping.perm().contains(MMUFlags::WRITE) {
                let address = mapping.start();
                let size = mapping.size();

                let mut cursor = self.vm_space.cursor(address)?;
                cursor.protect(size, |cprop| {
                    cprop.flags.remove(MMUFlags::WRITE);
                })?;
            }
        }

        Ok(Arc::new(Self {
            vm_space: Arc::new(VmSpace::new_user()),
            inner: RwLock::new(VmarInner { vm_mappings }),
        }))
    }
}

fn get_intersected_range(
    range1: &Range<VirtualAddress>,
    range2: &Range<VirtualAddress>,
) -> Range<VirtualAddress> {
    range1.start.max(range2.start)..range1.end.min(range2.end)
}
