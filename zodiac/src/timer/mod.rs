use alloc::{boxed::Box, vec::Vec};
use spin::Lazy;

use crate::{arch::context::TrapFrame, sync::CpuCell};

type TimerFn = Box<dyn Fn() + Sync + Send>;

static TIMER_CALLBACKS: Lazy<CpuCell<Vec<TimerFn>>> = Lazy::new(CpuCell::new);

pub fn register_callback_on_cpu<F>(func: F)
where
    F: Fn() + Sync + Send + 'static,
{
    TIMER_CALLBACKS.with_current_mut(|callbacks| {
        callbacks.push(Box::new(func));
    });
}

pub(crate) fn call_timer_callback_functions(_: &TrapFrame) {
    TIMER_CALLBACKS.with_current(|callbacks| {
        for callback in callbacks.iter() {
            (callback)();
        }
    });
}
