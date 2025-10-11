use ostd::{
    mm::{PageFlags, Vaddr, PAGE_SIZE},
    task::disable_preempt,
    Error,
};

use crate::mem::{align_down_by_page_size, Vmar};

impl Vmar {
    pub fn handle_page_fault(
        &self,
        vaddr: Vaddr,
        flags_required: PageFlags,
    ) -> Result<bool, Error> {
        let mut inner = self.inner.write();
        let mut handled = false;
        for mapping in inner.vm_mappings.iter_mut() {
            if mapping.contains(vaddr) {
                let flags = mapping.prop().flags;
                if !flags.contains(flags_required) {
                    continue;
                }

                handled = true;

                let start = mapping.start();
                let prop = mapping.prop();

                let vmo = mapping.vmo();
                if vmo.is_iomem() {
                    let end = start + mapping.size();

                    let (io_mem, base_offset) = vmo.into_iomem().unwrap();
                    self.vm_space
                        .cursor_mut(&disable_preempt(), &(start..end))?
                        .map_iomem(io_mem.clone(), prop, vmo.len(), base_offset);
                } else {
                    let (_, frame) = vmo.into_ram(vaddr - start)?.unwrap();

                    let start = align_down_by_page_size(vaddr);

                    self.vm_space
                        .cursor_mut(&disable_preempt(), &(start..start + PAGE_SIZE))?
                        .map(frame, prop);
                }
            }
        }
        Ok(handled)
    }
}
