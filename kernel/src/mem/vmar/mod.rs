use alloc::{sync::Arc, vec::Vec};
use ostd::{
    mm::{tlb::TlbFlushOp, PageFlags, PageProperty, Vaddr, VmSpace, PAGE_SIZE},
    sync::RwMutex,
    task::disable_preempt,
    Error,
};

use mapping::VmMapping;

use crate::mem::{align_down_by_page_size, align_up_by_page_size, Vmo};

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
    pub fn map(&self, addr: Vaddr, size: usize, prop: PageProperty) -> Result<(), Error> {
        let aligned = align_down_by_page_size(addr);
        let size = align_up_by_page_size(size + addr - aligned);
        
        let mut inner = self.inner.write();

        let vmo = Vmo::allocate_ram(size / PAGE_SIZE)?;

        let vm_mapping = VmMapping::new(vmo, aligned, size, prop);

        if inner
            .vm_mappings
            .iter()
            .any(|mapping| mapping.overlaps(&vm_mapping))
        {
            return Err(Error::AccessDenied);
        }

        inner.vm_mappings.push(vm_mapping);

        Ok(())
    }

    pub fn unmap(&self, addr: Vaddr, size: usize) -> Result<(), Error> {
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

    pub fn protect(&self, addr: Vaddr, size: usize, flags: PageFlags) -> Result<(), Error> {
        let mut inner = self.inner.write();

        for mapping in inner.vm_mappings.iter_mut() {
            if mapping.contains_range(addr, size) {
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
    pub fn deep_clone(&self) -> Result<Arc<Self>, Error> {
        let mut vm_mappings = Vec::new();
        for mapping in self.inner.read().vm_mappings.iter() {
            vm_mappings.push(mapping.clone()?);
        }

        Ok(Arc::new(Self {
            vm_space: Arc::new(VmSpace::new()),
            inner: RwMutex::new(VmarInner { vm_mappings }),
        }))
    }
}
