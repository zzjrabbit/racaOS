use alloc::{
    boxed::Box,
    sync::{Arc, Weak},
};
use spin::Mutex;

use crate::{error::RcResult, hal::int::IntFrame, object::KObjectBase};

use super::{process::Process, scheduler::SCHEDULER};

crate::kernel_object! {
    pub struct Thread {
        proc: Weak<Process> = Arc::downgrade(&Process::new()),
        kernel_stack: KernelStack = KernelStack::default(),
        inner: Mutex<ThreadInner> = Mutex::new(ThreadInner::new()),
    }

    fn new() {}

    fn related_koid(&self) -> crate::object::KoID {
        self.proc.upgrade().unwrap().id()
    }
}

pub struct ThreadInner {
    context: IntFrame,
    sleeping: bool,
}

impl ThreadInner {
    pub fn new() -> Self {
        Self {
            context: IntFrame::default(),
            sleeping: false,
        }
    }
}

impl Thread {
    /// Create a new thread.
    pub fn create(
        proc: &Arc<Process>,
        name: &str,
        entry: usize,
        stack: usize,
    ) -> RcResult<Arc<Self>> {
        let mut inner = ThreadInner::new();
        inner.context.rip = entry;
        inner.context.rsp = stack;
        inner.context.rflags = 0x200;
        inner.context.rdx = stack;
        inner.context.rdi = stack;
        let (code_selector, data_selector) = crate::hal::gdt::Selectors::get_user_segments();
        inner.context.cs = code_selector.0 as usize;
        inner.context.ss = data_selector.0 as usize;

        let thread = Arc::new(Thread {
            base: KObjectBase::with_name(name),
            proc: Arc::downgrade(proc),
            kernel_stack: KernelStack::default(),
            inner: Mutex::new(inner),
        });
        proc.add_thread(thread.clone())?;
        SCHEDULER.add_thread(&thread);
        Ok(thread)
    }
}

impl Thread {
    pub fn restore_context(&self, context: IntFrame) {
        self.inner.lock().context = context;
    }

    pub fn load_context(&self, context: &mut IntFrame) {
        *context = self.inner.lock().context.clone();
    }

    pub fn process(&self) -> Weak<Process> {
        self.proc.clone()
    }

    pub fn begin_sleep(&self) {
        self.inner.lock().sleeping = true;
    }

    pub fn wake_up(&self) {
        self.inner.lock().sleeping = false;
    }

    pub fn sleeping(&self) -> bool {
        self.inner.lock().sleeping
    }
}

impl Thread {
    pub fn kernel_stack(&self) -> usize {
        self.kernel_stack.end_address()
    }
}

const KERNEL_STACK_SIZE: usize = 64 * 1024;

pub struct KernelStack(Box<[u8]>);

impl Default for KernelStack {
    fn default() -> Self {
        Self(Box::from(alloc::vec![0; KERNEL_STACK_SIZE]))
    }
}

impl KernelStack {
    pub fn end_address(&self) -> usize {
        self.0.as_ptr_range().end as usize
    }
}
