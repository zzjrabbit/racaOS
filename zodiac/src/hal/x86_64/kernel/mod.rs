mod acpi;
mod apic;
pub mod irq;

pub use acpi::{reboot, shutdown};
pub(super) use acpi::ACPI;
pub(crate) use apic::end_of_interrupt;
pub(super) use apic::LAPIC;

pub(crate) fn init() {
    apic::init();
}

pub(crate) fn ap_init() {
    apic::ap_init();
}
