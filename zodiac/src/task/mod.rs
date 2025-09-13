use core::{
    any::Any,
    sync::atomic::{AtomicUsize, Ordering},
};

use alloc::{boxed::Box, sync::Arc, vec::Vec};
use spin::RwLock;

use crate::{
    ZodiacError,
    hal::{context::TrapFrame, cpu::Cpu},
    mem::VirtualAddress,
};

mod scheduler;
mod user;

pub use scheduler::*;
pub use user::*;

pub type TaskId = usize;

const KERNEL_STACK_SIZE: usize = 64 * 1024; // 64k

/// Task structure.
/// This is basically a container of context .
pub struct Task {
    inner: RwLock<TaskInner>,
    task_id: TaskId,
    _kernel_stack: Vec<u8>,
    kernel_stack_ptr: RwLock<VirtualAddress>,
    user_stack_ptr: RwLock<VirtualAddress>,
    data: Box<dyn Any + Sync + Send>,
}

struct TaskInner {
    task_state: TaskState,
    context: TrapFrame,
}

impl Task {
    /// Get the current task.
    pub fn current() -> Arc<Self> {
        current_task()
    }

    /// Spawn this task.
    pub fn spawn(self: &Arc<Self>) {
        sheduler_check();
        add_task(self.clone());
    }
}

impl Task {
    /// Get the task id of this task.
    pub fn task_id(&self) -> TaskId {
        self.task_id
    }

    /// Returns the task data.
    pub fn data(&self) -> &Box<dyn Any + Send + Sync> {
        &self.data
    }

    /// Get the task state of this task.
    pub fn task_state(&self) -> TaskState {
        self.inner.read().task_state
    }

    pub(crate) fn set_task_state(&self, state: TaskState) {
        self.inner.write().task_state = state;
    }

    pub(crate) fn context(&self) -> TrapFrame {
        self.inner.read().context.clone()
    }

    pub(crate) fn set_context(&self, context: TrapFrame) {
        self.inner.write().context = context;
    }

    pub(crate) fn kernel_stack(&self) -> VirtualAddress {
        *self.kernel_stack_ptr.read()
    }

    pub(crate) fn set_kernel_stack(&self, stack: VirtualAddress) {
        *self.kernel_stack_ptr.write() = stack;
    }

    pub(crate) fn user_stack(&self) -> VirtualAddress {
        *self.user_stack_ptr.read()
    }

    pub(crate) fn set_user_stack(&self, stack: VirtualAddress) {
        *self.user_stack_ptr.write() = stack;
    }
}

impl Task {
    /// Block this task.
    pub fn block(&self) {
        self.set_task_state(TaskState::Blocked);
        self.r#yield();
    }

    /// Yield the CPU to another task.
    pub fn r#yield(&self) {
        Cpu::current().trigger_schedule();
    }

    pub(crate) fn run(&self) {
        self.set_task_state(TaskState::RunningOn(Cpu::current()));
    }

    pub(crate) fn ready(&self) {
        self.set_task_state(TaskState::Ready);
    }
}

impl Task {
    /// Exit this task.
    pub fn exit(&self) -> ! {
        remove_task(self.task_id());
        self.set_task_state(TaskState::Dead);
        Cpu::current().trigger_schedule();
        loop {}
    }

    /// Kill this task.
    pub fn kill(&self) {
        remove_task(self.task_id());

        let cpu = if let TaskState::RunningOn(cpu) = self.task_state() {
            Some(cpu)
        } else {
            None
        };
        self.set_task_state(TaskState::Dead);

        if let Some(cpu) = cpu {
            cpu.trigger_schedule();
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TaskState {
    Ready,
    RunningOn(Cpu),
    Blocked,
    Dead,
}

impl TaskState {
    pub fn is_blocked(&self) -> bool {
        !matches!(self, Self::Ready | Self::RunningOn(_))
    }

    pub fn on_cpu(&self) -> Option<Cpu> {
        match self {
            TaskState::RunningOn(cpu) => Some(*cpu),
            _ => None,
        }
    }
}

/// Builder of a task.
pub struct TaskBuilder {
    kernel_stack_size: usize,
    entry: Option<fn() -> !>,
    data: Option<Box<dyn Any + Send + Sync>>,
}

impl Default for TaskBuilder {
    fn default() -> Self {
        Self {
            kernel_stack_size: KERNEL_STACK_SIZE,
            entry: None,
            data: None,
        }
    }
}

impl TaskBuilder {
    /// Set the entry point of the task.
    /// This is a necessary option.
    pub fn entry(mut self, entry: fn() -> !) -> Self {
        self.entry = Some(entry);
        self
    }

    /// Set the kernel stack size of the task.
    /// This is optional.
    pub fn kernel_stack_size(mut self, size: usize) -> Self {
        self.kernel_stack_size = size;
        self
    }

    /// Set the data of the task.
    /// You can use this to store process id and so on.
    pub fn data<T>(mut self, data: T) -> Self
    where
        T: Any + Send + Sync,
    {
        self.data = Some(Box::new(data));
        self
    }
}

static NEXT_TASK_ID: AtomicUsize = AtomicUsize::new(0);

impl TaskBuilder {
    /// Create the task.
    pub fn build(self) -> Result<Arc<Task>, ZodiacError> {
        let kernel_stack = alloc::vec![0; self.kernel_stack_size];
        let kernel_stack_ptr = kernel_stack.as_ptr() as VirtualAddress + self.kernel_stack_size;

        let entry = self.entry.ok_or(ZodiacError::ArgumentsNotEnough)?;

        let mut context = TrapFrame::default();
        context.init(&kernel_stack, entry as usize);

        let task = Arc::new(Task {
            task_id: NEXT_TASK_ID.fetch_add(1, Ordering::SeqCst),
            _kernel_stack: kernel_stack,
            kernel_stack_ptr: RwLock::new(kernel_stack_ptr),
            user_stack_ptr: RwLock::new(0),
            data: if let Some(data) = self.data {
                data
            } else {
                Box::new(())
            },
            inner: RwLock::new(TaskInner {
                context,
                task_state: TaskState::Ready,
            }),
        });

        Ok(task)
    }
}
