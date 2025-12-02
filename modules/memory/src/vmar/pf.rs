use errors::Result;
use mostd::mem::{MMUFlags, VirtualAddress};

use crate::{PAGE_SIZE, Vmar, align_down_by_page_size};

impl Vmar {
    pub fn handle_page_fault(
        &self,
        vaddr: VirtualAddress,
        perm_required: MMUFlags,
    ) -> Result<bool> {
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

                    let (io_mem, base_offset) = vmo.into_iomem().unwrap();
                    self.vm_space.cursor(start)?.map_iomem(
                        &io_mem,
                        prop,
                        base_offset,
                        vmo.len(),
                    )?;
                } else if perm_required.contains(MMUFlags::WRITE)
                    && !prop.flags.contains(MMUFlags::WRITE)
                {
                    log::info!("CoW");
                    // Perform CoW.
                    prop.flags |= MMUFlags::WRITE;
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

                        self.vm_space.cursor(start)?.unmap(PAGE_SIZE)?;
                        self.vm_space.cursor(start)?.map(&frame, prop)?;
                    }
                } else {
                    let (_, frame) = mapping.vmo().into_ram(vaddr - start)?.unwrap();

                    let start = align_down_by_page_size(vaddr);

                    let mut cursor = self.vm_space.cursor(start)?;
                    cursor.map(&frame, prop)?;
                }

                break;
            }
        }
        Ok(handled)
    }
}
