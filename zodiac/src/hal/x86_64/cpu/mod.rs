use alloc::vec::Vec;

use crate::{
    hal::{kernel::LAPIC, smp::MP_REQUEST},
    trap::Irq,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cpu(u32);

impl Cpu {
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

    pub fn all_cpus() -> Vec<Self> {
        let mut cpus = Vec::new();
        for cpu in MP_REQUEST.get_response().unwrap().cpus().iter() {
            cpus.push(Self(cpu.lapic_id));
        }
        cpus
    }
}
