use alloc::{sync::Arc, vec::Vec};
use errors::Result;
use mostd::mem::{MMUFlags, PageProperty, VirtualAddress, VmSpace};
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
    ) -> Result<()> {
        if size == 0 {
            return Ok(());
        }

        let aligned = align_down_by_page_size(addr);
        let size = align_up_by_page_size(size + addr - aligned);

        let mut inner = self.inner.write();

        let vmo = Vmo::allocate_ram(size / PAGE_SIZE)?;

        let vm_mapping = VmMapping::new(vmo, aligned, size, prop, prop.flags);

        if process_overlap {
            let mut new_mappings = Vec::new();
            let mut mappings_to_remove = Vec::new();
            for mapping in inner.vm_mappings.iter_mut() {
                if mapping.overlaps(&vm_mapping) {
                    let (new, to_remove) = mapping.make_not_overlap_with(&vm_mapping);
                    if let Some(new) = new {
                        new_mappings.push(new);
                    }
                    if to_remove {
                        mappings_to_remove.push(mapping.start());
                    }
                }
            }

            mappings_to_remove.sort();
            mappings_to_remove.dedup();
            for start in mappings_to_remove {
                inner.vm_mappings.retain(|mapping| mapping.start() != start);
            }

            inner.vm_mappings.extend(new_mappings);
        }

        inner.vm_mappings.push(vm_mapping);

        Ok(())
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

        let mut inner = self.inner.write();

        for mapping in inner.vm_mappings.iter_mut() {
            if mapping.contains_range(addr, size) {
                mapping.set_prop({
                    let mut prop = mapping.prop();
                    prop.flags |= flags;
                    prop
                });
                mapping.set_perm({
                    let mut perm = mapping.perm();
                    perm.insert(flags);
                    perm
                });

                let aligned = align_down_by_page_size(addr);
                let size = align_up_by_page_size(size + addr - aligned);

                let mut cursor = self.vm_space.cursor(aligned)?;
                cursor.protect(size, |cprop| {
                    cprop.flags.insert(flags);
                })?;

                let mut prop = mapping.prop();
                prop.flags |= flags;
                mapping.set_prop(prop);
            }
        }

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

                log::debug!("OK {address:x} {size:x}");
                let mut cursor = self.vm_space.cursor(address)?;
                log::debug!("OK");
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
