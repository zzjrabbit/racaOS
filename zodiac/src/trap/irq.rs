use alloc::{collections::btree_map::BTreeMap, vec::Vec};
use spin::RwLock;

use crate::hal::{
    context::TrapFrame,
    kernel::irq::{MAX_IRQ_NUM, MIN_IRQ_NUM},
};

/// Handler for an IRQ.
pub type IrqHandler = fn(&mut TrapFrame);

pub(crate) struct IrqManager {
    inner: RwLock<IrqManagerInner>,
}

struct IrqManagerInner {
    irqs: BTreeMap<Irq, IrqHandler>,
    available: Vec<Irq>,
}

impl IrqManager {
    pub(super) fn new() -> Self {
        let mut available_irqs = Vec::new();

        let mut current = MIN_IRQ_NUM as usize;
        while current <= MAX_IRQ_NUM as usize {
            available_irqs.push(Irq(current as u8));
            current += 1;
        }

        Self {
            inner: RwLock::new(IrqManagerInner {
                irqs: BTreeMap::new(),
                available: available_irqs,
            }),
        }
    }
}

impl IrqManager {
    pub(super) fn allocate_irq(&self, handler: IrqHandler) -> Option<Irq> {
        let mut inner = self.inner.write();

        let irq = inner.available.pop()?;
        inner.irqs.insert(irq, handler);
        Some(irq)
    }

    pub(super) fn allocate_specific_irq(&self, irq_id: u8, handler: IrqHandler) -> Option<Irq> {
        if irq_id > MAX_IRQ_NUM - MIN_IRQ_NUM {
            return None;
        }

        let mut inner = self.inner.write();

        let irq = Irq(irq_id + MIN_IRQ_NUM);

        if inner.irqs.contains_key(&irq) {
            return None;
        }

        inner.available.retain(|&i| i != irq);
        inner.irqs.insert(irq, handler);
        Some(irq)
    }

    pub(crate) fn handle_irq(&self, frame: &mut TrapFrame) {
        if let Some(handler) = self.inner.read().irqs.get(&Irq(frame.int_num as u8)) {
            crate::hal::kernel::end_of_interrupt();
            handler(frame);
        }
    }
}

/// Safe wrapper for IRQs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Irq(u8);

impl Irq {
    pub(crate) fn as_int_vector(&self) -> u8 {
        self.0
    }

    /// Allocate a specific IRQ.
    pub fn allocate_specific(irq_id: u8, handler: IrqHandler) -> Option<Irq> {
        super::IRQ_MANAGER.allocate_specific_irq(irq_id, handler)
    }

    /// Allocate an IRQ.
    pub fn allocate(handler: IrqHandler) -> Option<Irq> {
        super::IRQ_MANAGER.allocate_irq(handler)
    }

    /// Create an IRQ from a vector.
    pub fn from_vector(vector: u8) -> Self {
        Irq(vector)
    }
}
