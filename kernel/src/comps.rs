use alloc::vec::Vec;
use component::{component_list, ComponentRegistry};

pub fn components() -> Vec<&'static ComponentRegistry> {
    component_list!(
        logger, memory, block, filesystem, task, pci, fat, driver, network, nvme, ramdisk, terminal
    )
}
