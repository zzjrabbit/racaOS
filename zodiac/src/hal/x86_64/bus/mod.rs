mod pcie;

pub use pcie::*;
use spin::Lazy;

pub fn init() {
    Lazy::force(&PCI_DEVICES);
}
