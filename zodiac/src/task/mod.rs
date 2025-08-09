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

pub use scheduler::*;

pub type TaskId = usize;

const KERNEL_STACK_SIZE: usize = 64 * 1024; // 64k

/// Task structure.
/// This is basically a container of context .
pub struct Task {
    inner: RwLock<TaskInner>,
    task_id: TaskId,
    kernel_stack: Vec<u8>,
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
        self.kernel_stack.as_ptr() as VirtualAddress + self.kernel_stack.len()
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
        Cpu::current().trigger_save_context();
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

        self.r#yield();
        unreachable!()
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
            cpu.trigger_save_context();
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
        match self {
            Self::Ready | Self::RunningOn(_) => false,
            _ => true,
        }
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
    user_mode: bool,
    entry: Option<usize>,
    stack: Option<usize>,
    data: Option<Box<dyn Any + Send + Sync>>,
}

impl Default for TaskBuilder {
    fn default() -> Self {
        Self {
            kernel_stack_size: KERNEL_STACK_SIZE,
            user_mode: true,
            entry: None,
            stack: None,
            data: None,
        }
    }
}

impl TaskBuilder {
    /// Set the entry point of the task.
    /// This is a necessary option.
    pub fn entry(mut self, entry: fn() -> !) -> Self {
        self.entry = Some(entry as usize);
        self
    }

    /// Set the stack of the task.
    /// This is a necessary option for user tasks, and it does nothing for kernel tasks.
    pub fn stack(mut self, stack: usize) -> Self {
        self.stack = Some(stack);
        self
    }

    /// Set the kernel stack size of the task.
    /// This is optional.
    pub fn kernel_stack_size(mut self, size: usize) -> Self {
        self.kernel_stack_size = size;
        self
    }

    /// Make the task in kernel mode.
    pub fn kernel_mode(mut self) -> Self {
        self.user_mode = false;
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

        let entry = self.entry.ok_or(ZodiacError::ArgumentsNotEnough)?;
        let stack = if self.user_mode {
            self.stack.ok_or(ZodiacError::ArgumentsNotEnough)?
        } else {
            0
        };

        let mut context = TrapFrame::default();
        context.init(&kernel_stack, entry, stack, self.user_mode);

        let task = Arc::new(Task {
            task_id: NEXT_TASK_ID.fetch_add(1, Ordering::SeqCst),
            kernel_stack,
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
