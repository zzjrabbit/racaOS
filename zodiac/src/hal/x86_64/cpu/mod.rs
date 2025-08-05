use alloc::vec::Vec;

use crate::{
    hal::{
        kernel::LAPIC,
        smp::{BSP_LAPIC_ID, MP_REQUEST},
    },
    trap::Irq,
};

/// A structure to access CPUs in a safe way.
/// In x86_64, this is basically a wrapper of Local APIC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cpu(u32);

impl Cpu {
    pub(crate) fn new(id: u32) -> Self {
        Self(id)
    }

    /// Get the BSP CPU.
    pub fn bsp() -> Self {
        Self(*BSP_LAPIC_ID)
    }

    /// Get the current CPU.
    pub fn current() -> Self {
        unsafe { Self(LAPIC.lock().id()) }
    }

    pub(crate) fn id(&self) -> u32 {
        self.0
    }

    /// Send an inter-processor interrupt to the CPU.
    pub fn send_ipi(&self, irq: Irq) {
        unsafe {
            LAPIC.lock().send_ipi(irq.as_int_vector(), self.0);
        }
    }

    /// Get all CPUs.
    pub fn all() -> Vec<Self> {
        let mut cpus = Vec::new();
        for cpu in MP_REQUEST.get_response().unwrap().cpus().iter() {
            cpus.push(Self::new(cpu.lapic_id));
        }
        cpus
    }
}
