use crate::{arch::context::TrapFrame, timer::call_timer_callback_functions};

pub fn call_timer_callbacks(frame: &mut TrapFrame) {
    call_timer_callback_functions(frame);
}
