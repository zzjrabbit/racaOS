use ostd::arch::cpu::context::FpuContext;

use crate::UserThreadData;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(super) enum FpuState {
    Activated,
    Loaded,
    #[default]
    Unloaded,
}

pub struct Fpu<'a>(&'a UserThreadData);

impl<'a> Fpu<'a> {
    pub(super) fn new(data: &'a UserThreadData) -> Self {
        Fpu(data)
    }
}

impl<'a> Fpu<'a> {
    pub fn activate(&self) {
        match *self.0.fpu_state.read() {
            FpuState::Activated => return,
            FpuState::Loaded => {}
            FpuState::Unloaded => self.0.fpu_context.write().load(),
        }
        *self.0.fpu_state.write() = FpuState::Activated;
    }

    pub fn deactivate(&self) {
        let mut fpu_state = self.0.fpu_state.write();
        if matches!(*fpu_state, FpuState::Activated) {
            *fpu_state = FpuState::Loaded;
        }
    }

    pub fn clone_context(&self) -> FpuContext {
        match *self.0.fpu_state.read() {
            FpuState::Activated | FpuState::Loaded => {
                let mut fpu_context = self.0.fpu_context.write();
                fpu_context.save();
                fpu_context.clone()
            }
            FpuState::Unloaded => self.0.fpu_context.read().clone(),
        }
    }

    pub fn set_context(&self, context: FpuContext) {
        *self.0.fpu_context.write() = context;
        *self.0.fpu_state.write() = FpuState::Unloaded;
    }

    pub fn before_schedule(&self) {
        match *self.0.fpu_state.read() {
            FpuState::Activated => {
                self.0.fpu_context.write().save();
            }
            FpuState::Loaded => {
                self.0.fpu_context.write().save();
                *self.0.fpu_state.write() = FpuState::Unloaded;
            }
            FpuState::Unloaded => {}
        }
    }

    pub fn after_schedule(&self) {
        if matches!(*self.0.fpu_state.read(), FpuState::Activated) {
            self.0.fpu_context.write().load();
        }
    }
}
