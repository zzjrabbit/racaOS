use crate::hal::context::{CpuException, RawUserContext, TrapFrame};

#[derive(Debug)]
pub enum ReturnReason {
    Exception(CpuException),
    Syscall,
    KernelEvent,
}

pub struct UserContext {
    inner: RawUserContext,
}

impl UserContext {
    pub fn new(entry: usize, stack: usize) -> Self {
        Self {
            inner: RawUserContext::new(entry, stack),
        }
    }

    pub fn excute<F>(&mut self, has_kernel_event: F) -> ReturnReason
    where
        F: FnMut() -> bool,
    {
        self.inner.execute(has_kernel_event)
    }

    pub fn trap_frame(&mut self) -> &mut TrapFrame {
        self.inner.trap_frame()
    }
}
