use alloc::{boxed::Box, vec::Vec};
use spin::RwLock;

use crate::hal::{context::TrapFrame, disable_interrupts, enable_interrupts, interrupts_enabled};

type InterruptCallback = Box<dyn Fn(&mut TrapFrame) + Sync + Send>;

pub(crate) static TIMER_CALLBACKS: RwLock<Vec<InterruptCallback>> = RwLock::new(Vec::new());

pub fn register_callback<F>(func: F)
where
    F: Fn(&mut TrapFrame) + Sync + Send + 'static,
{
    let int_enabled = interrupts_enabled();
    disable_interrupts();
    TIMER_CALLBACKS.write().push(Box::new(func));
    if int_enabled {
        enable_interrupts();
    }
}
