use errors::Result;
use ostd::{
    mm::{PAGE_SIZE, PageFlags, Vaddr, tlb::TlbFlushOp},
    task::disable_preempt,
};

use crate::{Vmar, align_down_by_page_size};

impl Vmar {
    pub fn handle_page_fault(&self, vaddr: Vaddr, perm_required: PageFlags) -> Result<bool> {
        let mut inner = self.inner.write();
        let mut handled = false;
        for mapping in inner.vm_mappings.iter_mut() {
            if mapping.contains(vaddr) {
                let perm = mapping.perm();
                if !perm.contains(perm_required) {
                    log::warn!(
                        "Page fault at {:x} with required permissions {:?}, but got {:?}",
                        vaddr,
                        perm_required,
                        perm
                    );
                    continue;
                }

                handled = true;

                let mut prop = mapping.prop();
                let start = mapping.start();

                if mapping.vmo().is_iomem() {
                    let vmo = mapping.vmo().clone();
                    let end = start + mapping.size();

                    let (io_mem, base_offset) = vmo.into_iomem().unwrap();
                    self.vm_space
                        .cursor_mut(&disable_preempt(), &(start..end))?
                        .map_iomem(io_mem.clone(), prop, vmo.len(), base_offset);
                } else if perm_required.contains(PageFlags::W) && !prop.flags.contains(PageFlags::W)
                {
                    log::info!("CoW");
                    // Perform CoW.
                    prop.flags |= PageFlags::W;
                    mapping.set_prop(prop);

                    *mapping.vmo_mut() = mapping.vmo().deep_clone()?;

                    let vmo = mapping.vmo().clone();
                    let count = vmo.len() / PAGE_SIZE;

                    for id in 0..count {
                        if !vmo.commited(id) {
                            continue;
                        }

                        let start = start + id * PAGE_SIZE;

                        let (_, frame) = vmo.into_ram(id * PAGE_SIZE)?.unwrap();

                        let guard = disable_preempt();

                        self.vm_space
                            .cursor_mut(&guard, &(start..start + PAGE_SIZE))?
                            .unmap(PAGE_SIZE);
                        self.vm_space
                            .cursor_mut(&guard, &(start..start + PAGE_SIZE))?
                            .map(frame, prop);
                    }
                } else {
                    let (_, frame) = mapping.vmo().into_ram(vaddr - start)?.unwrap();

                    let start = align_down_by_page_size(vaddr);

                    let guard = disable_preempt();
                    let mut cursor = self
                        .vm_space
                        .cursor_mut(&guard, &(start..start + PAGE_SIZE))?;
                    cursor.map(frame, prop);

                    let flusher = cursor.flusher();
                    flusher.issue_tlb_flush(TlbFlushOp::for_range(start..start + PAGE_SIZE));
                    flusher.dispatch_tlb_flush();
                }

                break;
            }
        }
        Ok(handled)
    }
}
