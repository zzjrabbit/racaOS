use alloc::{sync::Arc, vec::Vec};
use errors::Result;
use ostd::{
    mm::{PAGE_SIZE, PageFlags, PageProperty, Vaddr, VmSpace, tlb::TlbFlushOp},
    sync::RwMutex,
    task::disable_preempt,
};

use mapping::VmMapping;

use crate::{Vmo, align_down_by_page_size, align_up_by_page_size};

mod mapping;
mod pf;
mod rw;

#[derive(Debug)]
pub struct Vmar {
    vm_space: Arc<VmSpace>,
    inner: RwMutex<VmarInner>,
}

#[derive(Debug)]
struct VmarInner {
    vm_mappings: Vec<VmMapping>,
}

impl Vmar {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            vm_space: Arc::new(VmSpace::new()),
            inner: RwMutex::new(VmarInner {
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
    pub fn map(&self, addr: Vaddr, size: usize, prop: PageProperty) -> Result<()> {
        if size == 0 {
            return Ok(());
        }
        
        let aligned = align_down_by_page_size(addr);
        let size = align_up_by_page_size(size + addr - aligned);

        let mut inner = self.inner.write();

        let vmo = Vmo::allocate_ram(size / PAGE_SIZE)?;

        let vm_mapping = VmMapping::new(vmo, aligned, size, prop, prop.flags);

        let mut new_mappings = Vec::new();
        let mut overlap = false;

        for mapping in inner.vm_mappings.iter() {
            if mapping.overlaps(&vm_mapping) {
                overlap = true;
                let pre_len = mapping.start() as isize - aligned as isize;
                let post_len =
                    (aligned + size) as isize - (mapping.start() + mapping.size()) as isize;
                if pre_len > 0 {
                    new_mappings.push(VmMapping::new(
                        Vmo::allocate_ram(pre_len as usize / PAGE_SIZE)?,
                        aligned,
                        pre_len as usize,
                        prop,
                        prop.flags,
                    ));
                }
                if post_len > 0 {
                    new_mappings.push(VmMapping::new(
                        Vmo::allocate_ram(post_len as usize / PAGE_SIZE)?,
                        aligned + size - post_len as usize,
                        post_len as usize,
                        prop,
                        prop.flags,
                    ));
                }
                break;
            }
        }

        if !overlap {
            inner.vm_mappings.push(vm_mapping);
        } else {
            inner.vm_mappings.extend(new_mappings);
        }

        Ok(())
    }

    pub fn unmap(&self, addr: Vaddr, size: usize) -> Result<()> {
        if size == 0 {
            return Ok(());
        }
        
        let mut inner = self.inner.write();

        for mapping in inner.vm_mappings.iter_mut() {
            if mapping.contains_range(addr, size) {
                let aligned = align_down_by_page_size(addr);
                let size = align_up_by_page_size(size + addr - aligned);

                let guard = disable_preempt();
                let mut cursor = self
                    .vm_space
                    .cursor_mut(&guard, &(aligned..aligned + size))?;
                cursor.unmap(size);
                cursor
                    .flusher()
                    .issue_tlb_flush(TlbFlushOp::for_range(aligned..aligned + size));
                cursor.flusher().dispatch_tlb_flush();
            }
        }

        inner
            .vm_mappings
            .retain(|mapping| !mapping.contains_range(addr, size));

        Ok(())
    }

    pub fn protect(&self, addr: Vaddr, size: usize, flags: PageFlags) -> Result<()> {
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

                let guard = disable_preempt();
                let mut cursor = self
                    .vm_space
                    .cursor_mut(&guard, &(aligned..aligned + size))?;
                cursor.protect_next(size, |cprop, _| {
                    cprop.insert(flags);
                });
                cursor
                    .flusher()
                    .issue_tlb_flush(TlbFlushOp::for_range(aligned..aligned + size));
                cursor.flusher().dispatch_tlb_flush();

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
        let guard = disable_preempt();
        
        let mut vm_mappings = Vec::new();
        for mapping in self.inner.write().vm_mappings.iter_mut() {
            vm_mappings.push(mapping.clone()?);
            if mapping.perm().contains(PageFlags::W) {
                let address = mapping.start();
                let size = mapping.size();
                
                log::debug!("OK {address:x} {size:x}");
                let mut cursor = self
                    .vm_space
                    .cursor_mut(&guard, &(address..address + size))?;
                log::debug!("OK");
                cursor.protect_next(size, |cprop, _| {
                    cprop.remove(PageFlags::W);
                });
                cursor
                    .flusher()
                    .issue_tlb_flush(TlbFlushOp::for_range(address..address + size));
                cursor.flusher().dispatch_tlb_flush();
            }
        }

        Ok(Arc::new(Self {
            vm_space: Arc::new(VmSpace::new()),
            inner: RwMutex::new(VmarInner { vm_mappings }),
        }))
    }
}
