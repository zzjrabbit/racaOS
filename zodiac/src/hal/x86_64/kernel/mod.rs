mod acpi;
mod apic;
pub mod irq;

pub(super) use acpi::ACPI;
pub use acpi::{reboot, shutdown};
pub(crate) use apic::end_of_interrupt;
pub(super) use apic::{LAPIC, apic_timer_irq};

pub(crate) fn init() {
    apic::init();
}

pub(crate) fn ap_init() {
    apic::ap_init();
}
