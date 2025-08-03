use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};
use spin::RwLock;

use crate::{
    ZodiacError,
    hal::{context::TrapFrame, cpu::Cpu},
    mem::VirtualAddress,
    task::{Process, add_thread},
};

pub type ThreadId = usize;

const KERNEL_STACK_SIZE: usize = 4 * 1024; // 4k

pub struct Thread {
    inner: RwLock<ThreadInner>,
    thread_id: ThreadId,
    process: Weak<Process>,
    _kernel_stack: Vec<u8>,
}

struct ThreadInner {
    kernel_stack_pointer: VirtualAddress,
    thread_state: ThreadState,
}

impl Thread {
    pub fn spawn(self: &Arc<Self>) {
        add_thread(self.clone());
    }
}

impl Thread {
    pub fn process(&self) -> Option<Arc<Process>> {
        self.process.upgrade()
    }

    pub fn thread_id(&self) -> ThreadId {
        self.thread_id
    }

    pub fn thread_state(&self) -> ThreadState {
        self.inner.read().thread_state
    }

    pub fn set_thread_state(&self, state: ThreadState) {
        self.inner.write().thread_state = state;
    }

    pub(crate) fn kernel_stack_pointer(&self) -> VirtualAddress {
        self.inner.read().kernel_stack_pointer
    }
    
    pub(crate) fn set_kernel_stack_pointer(&self, pointer: usize) {
        self.inner.write().kernel_stack_pointer = pointer;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ThreadState {
    Ready,
    RunningOn(Cpu),
    Blocked,
}

impl ThreadState {
    pub fn is_blocked(&self) -> bool {
        matches!(self, ThreadState::Blocked)
    }

    pub fn on_cpu(&self) -> Option<Cpu> {
        match self {
            ThreadState::RunningOn(cpu) => Some(*cpu),
            _ => None,
        }
    }
}

pub struct ThreadBuilder {
    process: Option<Arc<Process>>,
    kernel_stack_size: usize,
    user_mode: bool,
    entry: Option<usize>,
    stack: Option<usize>,
}

impl Default for ThreadBuilder {
    fn default() -> Self {
        Self {
            process: None,
            kernel_stack_size: KERNEL_STACK_SIZE,
            user_mode: true,
            entry: None,
            stack: None,
        }
    }
}

impl ThreadBuilder {
    pub fn entry(mut self, entry: fn() -> !) -> Self {
        self.entry = Some(entry as usize);
        self
    }

    pub fn stack(mut self, stack: usize) -> Self {
        self.stack = Some(stack);
        self
    }

    pub fn process(mut self, process: Arc<Process>) -> Self {
        self.process = Some(process);
        self
    }

    pub fn kernel_stack_size(mut self, size: usize) -> Self {
        self.kernel_stack_size = size;
        self
    }

    pub fn kernel_mode(mut self) -> Self {
        self.user_mode = false;
        self
    }
}

impl ThreadBuilder {
    pub fn build(self) -> Result<Arc<Thread>, ZodiacError> {
        static NEXT_THREAD_ID: AtomicUsize = AtomicUsize::new(0);

        let mut kernel_stack = alloc::vec![0; self.kernel_stack_size];

        let process = self.process.ok_or(ZodiacError::ArgumentsNotEnough)?;
        let entry = self.entry.ok_or(ZodiacError::ArgumentsNotEnough)?;
        let stack = if self.user_mode {
            self.stack.ok_or(ZodiacError::ArgumentsNotEnough)?
        } else {
            0
        };

        let kernel_stack_pointer = TrapFrame::init_in(&mut kernel_stack, entry, stack, self.user_mode);

        let thread = Arc::new(Thread {
            process: Arc::downgrade(&process),
            thread_id: NEXT_THREAD_ID.fetch_add(1, Ordering::SeqCst),
            _kernel_stack: kernel_stack,
            inner: RwLock::new(ThreadInner {
                kernel_stack_pointer,
                thread_state: ThreadState::Ready,
            }),
        });

        process.add_thread(thread.clone());
        Ok(thread)
    }
}
