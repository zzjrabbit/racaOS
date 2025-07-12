use alloc::{
    boxed::Box,
    sync::{Arc, Weak},
};
use spin::Mutex;

use crate::{
    error::RcResult,
    hal::{driver::apic::LAPIC, int::IntFrame},
    mm::VirtualMemory,
    object::{KObjectBase, KernelObject, Signal},
};

use super::{process::Process, scheduler::SCHEDULER};

crate::kernel_object! {
    pub struct Thread {
        proc: Weak<Process>,
        stack: Arc<VirtualMemory>,
        kernel_stack: KernelStack,
        inner: Mutex<ThreadInner>,
    }

    fn related_koid(&self) -> crate::object::KoID {
        self.proc.upgrade().unwrap().id()
    }
}

pub struct ThreadInner {
    context: IntFrame,
    state: ThreadState,
}

impl ThreadInner {
    pub fn new() -> Self {
        Self {
            context: IntFrame::default(),
            state: ThreadState::Ready,
        }
    }
}

impl Thread {
    /// Create a new thread.
    pub fn create(
        proc: &Arc<Process>,
        name: &str,
        entry: usize,
        stack: Arc<VirtualMemory>,
    ) -> RcResult<Arc<Self>> {
        let stack_start = stack.start_address() + stack.len();

        let mut inner = ThreadInner::new();
        inner.context.rip = entry;
        inner.context.rsp = stack_start;
        inner.context.rflags = 0x200;
        inner.context.rdx = stack_start;
        inner.context.rdi = stack_start;
        let (code_selector, data_selector) = crate::hal::gdt::Selectors::get_user_segments();
        inner.context.cs = code_selector.0 as usize;
        inner.context.ss = data_selector.0 as usize;

        let thread = Arc::new(Thread {
            base: KObjectBase::with_name(name),
            stack: stack.clone(),
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

    pub fn set_state(&self, state: ThreadState) {
        self.inner.lock().state = state;
    }

    pub fn state(&self) -> ThreadState {
        self.inner.lock().state
    }
}

impl Thread {
    pub fn exit(&self) { 
        self.set_signal(Signal::TASK_DEAD);
        self.set_state(ThreadState::Dead);
        SCHEDULER.remove_thread(self.id());
        self.process().upgrade().unwrap().remove_thread(self.id());
    }

    pub fn kill(&self) {
        let possible_running_on = match self.state() {
            ThreadState::RunningOn(cpu_id) => Some(cpu_id),
            _ => None,
        };

        self.exit();

        possible_running_on.and_then(|cpu_id| {
            if cpu_id == unsafe { LAPIC.lock().id() } {
                return None;
            }
            unsafe {
                LAPIC.lock().send_ipi(0x21, cpu_id);
            }
            Some(())
        });
    }
}

impl Thread {
    pub fn kernel_stack(&self) -> usize {
        self.kernel_stack.end_address()
    }

    pub fn stack(&self) -> Arc<VirtualMemory> {
        self.stack.clone()
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ThreadState {
    RunningOn(u32),
    Ready,
    Blocked,
    Dead,
}

impl ThreadState {
    pub fn running_on(&self) -> Option<u32> {
        match self {
            Self::RunningOn(cpu_id) => Some(*cpu_id),
            _ => None,
        }
    }

    pub fn is_awake(&self) -> bool {
        match self {
            Self::RunningOn(_) | Self::Ready => true,
            _ => false,
        }
    }
}
