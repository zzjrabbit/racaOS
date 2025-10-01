use alloc::vec::Vec;
use component::{component_list, ComponentRegistry};

pub fn components() -> Vec<&'static ComponentRegistry> {
    component_list!(terminal, logger, pci)
}
