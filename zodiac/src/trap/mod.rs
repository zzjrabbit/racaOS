use irq::IrqManager;
use spin::Lazy;

pub use crate::trap::irq::{Irq, IrqHandler};

mod irq;

pub(crate) static IRQ_MANAGER: Lazy<IrqManager> = Lazy::new(IrqManager::new);
