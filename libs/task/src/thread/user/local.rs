use core::cell::RefCell;

use ostd::{arch::cpu::context::FpuContext, task::CurrentTask};

use crate::{Fpu, thread::user::FpuState};

pub struct ThreadLocal {
    fpu_context: RefCell<FpuContext>,
    fpu_state: RefCell<FpuState>,
}

impl ThreadLocal {
    pub(super) fn new() -> Self {
        Self {
            fpu_context: RefCell::new(FpuContext::new()),
            fpu_state: RefCell::new(FpuState::default()),
        }
    }

    pub(super) fn fpu_context(&self) -> &RefCell<FpuContext> {
        &self.fpu_context
    }

    pub(super) fn fpu_state(&self) -> &RefCell<FpuState> {
        &self.fpu_state
    }
}

impl ThreadLocal {
    pub fn fpu(&self) -> Fpu {
        Fpu::new(self)
    }
}

pub trait AsThreadLocal {
    fn as_thread_local(&self) -> Option<&ThreadLocal>;
}

impl AsThreadLocal for CurrentTask {
    fn as_thread_local(&self) -> Option<&ThreadLocal> {
        self.local_data().downcast_ref::<ThreadLocal>()
    }
}
