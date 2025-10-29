use ostd::arch::cpu::context::FpuContext;

use crate::ThreadLocal;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(super) enum FpuState {
    Activated,
    Loaded,
    #[default]
    Unloaded,
}

pub struct Fpu<'a>(&'a ThreadLocal);

impl<'a> Fpu<'a> {
    pub(super) fn new(data: &'a ThreadLocal) -> Self {
        Fpu(data)
    }
}

impl<'a> Fpu<'a> {
    pub fn activate(&self) {
        match *self.0.fpu_state().borrow() {
            FpuState::Activated => return,
            FpuState::Loaded => {}
            FpuState::Unloaded => self.0.fpu_context().borrow_mut().load(),
        }
        self.0.fpu_state().replace(FpuState::Activated);
    }

    pub fn deactivate(&self) {
        let mut fpu_state = self.0.fpu_state().borrow_mut();
        if matches!(*fpu_state, FpuState::Activated) {
            *fpu_state = FpuState::Loaded;
        }
    }

    pub fn clone_context(&self) -> FpuContext {
        match *self.0.fpu_state().borrow() {
            FpuState::Activated | FpuState::Loaded => {
                let mut fpu_context = self.0.fpu_context().borrow_mut();
                fpu_context.save();
                fpu_context.clone()
            }
            FpuState::Unloaded => self.0.fpu_context().borrow().clone(),
        }
    }

    pub fn set_context(&self, context: FpuContext) {
        *self.0.fpu_context().borrow_mut() = context;
        *self.0.fpu_state().borrow_mut() = FpuState::Unloaded;
    }

    pub fn before_schedule(&self) {
        let fpu_state = *self.0.fpu_state().borrow();
        match fpu_state {
            FpuState::Activated => {
                self.0.fpu_context().borrow_mut().save();
            }
            FpuState::Loaded => {
                self.0.fpu_context().borrow_mut().save();
                *self.0.fpu_state().borrow_mut() = FpuState::Unloaded;
            }
            FpuState::Unloaded => {}
        }
    }

    pub fn after_schedule(&self) {
        if matches!(*self.0.fpu_state().borrow(), FpuState::Activated) {
            self.0.fpu_context().borrow_mut().load();
        }
    }
}
