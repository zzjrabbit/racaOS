mod acpi;
mod apic;
pub mod irq;

pub(super) use acpi::ACPI;
pub use acpi::{reboot, shutdown};
pub(super) use apic::LAPIC;
pub(crate) use apic::end_of_interrupt;

pub(crate) fn init() {
    apic::init();
}

pub(crate) fn ap_init() {
    apic::ap_init();
}
