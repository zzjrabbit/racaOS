use alloc::vec::Vec;

use crate::{
    hal::{
        kernel::LAPIC,
        smp::{BSP_LAPIC_ID, MP_REQUEST},
    },
    trap::Irq,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cpu(u32);

impl Cpu {
    pub fn new(id: u32) -> Self {
        Self(id)
    }

    pub fn bsp() -> Self {
        Self(*BSP_LAPIC_ID)
    }

    pub fn current() -> Self {
        unsafe { Self(LAPIC.lock().id()) }
    }

    pub fn id(&self) -> u32 {
        self.0
    }

    pub fn send_ipi(&self, irq: Irq) {
        unsafe {
            LAPIC.lock().send_ipi(irq.as_int_vector(), self.0);
        }
    }

    pub fn all() -> Vec<Self> {
        let mut cpus = Vec::new();
        for cpu in MP_REQUEST.get_response().unwrap().cpus().iter() {
            cpus.push(Self::new(cpu.lapic_id));
        }
        cpus
    }
}
