use ostd::{Error, mm::{PAGE_SIZE, Vaddr}, task::disable_preempt};

use crate::mem::Vmar;

impl Vmar {
    pub fn handle_page_fault(&self, vaddr: Vaddr) -> Result<bool, Error> {
        let mut inner = self.inner.write();
        let mut handled = false;
        for mapping in inner.vm_mappings.iter_mut() {
            if mapping.contains(vaddr) && !mapping.mapped() {
                handled = true;
                mapping.map();

                let start = mapping.start();
                let prop = mapping.prop();

                for (id, frame) in mapping.frames().iter().enumerate() {
                    let start = start + id * PAGE_SIZE;
                    let end = start + PAGE_SIZE;

                    self.vm_space
                        .cursor_mut(&disable_preempt(), &(start..end))?
                        .map(frame.clone(), prop);
                }
            }
        }
        Ok(handled)
    }
}
