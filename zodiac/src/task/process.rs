use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{sync::Arc, vec::Vec};
use spin::{Lazy, RwLock};

use crate::{
    hal::{
        context::TrapFrame, cpu::Cpu, trap::{change_context_save_action, ContextSaveAction}
    }, mem::VirtualMemorySpace, task::{remove_thread, Thread, ThreadId, ThreadState}, ZodiacError
};

pub type ProcessId = usize;

/// Process structure.
/// This is basically a container of resources.
pub struct Process {
    process_id: ProcessId,
    vm_space: VirtualMemorySpace,
    inner: RwLock<ProcessInner>,
}

struct ProcessInner {
    threads: Vec<Arc<Thread>>,
    is_child_process: bool,
}

impl Process {
    pub(crate) fn new_kernel() -> Arc<Self> {
        Arc::new(Self {
            process_id: 0,
            vm_space: VirtualMemorySpace::new_user(),
            inner: RwLock::new(ProcessInner {
                threads: Vec::new(),
                is_child_process: false,
            }),
        })
    }
    
    pub(crate) fn fork_impl(&self, context: TrapFrame, cow: bool) -> Arc<Self> {
        Arc::new(Self {
            process_id: NEXT_PROCESS_ID.fetch_add(1, Ordering::SeqCst),
            vm_space: self.vm_space.deep_copy(cow),
            inner: RwLock::new(ProcessInner {
                threads: Vec::new(),
                is_child_process: true,
            }),
        })
    }

    // Return the kernel process.
    pub fn kernel() -> Arc<Self> {
        static KERNEL: Lazy<Arc<Process>> = Lazy::new(Process::new_kernel);
        KERNEL.clone()
    }
}

impl Process {
    /// Return the process ID.
    pub fn process_id(&self) -> ProcessId {
        self.process_id
    }

    /// Add a thread to the process.
    pub fn add_thread(&self, thread: Arc<Thread>) {
        self.inner.write().threads.push(thread);
    }

    /// Remove a thread from the process.
    pub fn remove_thread(&self, thread_id: ThreadId) {
        self.inner
            .write()
            .threads
            .retain(|thread| thread.thread_id() != thread_id);
    }

    /// Return the virtual memory space of the process.
    pub fn vm_space(&self) -> &VirtualMemorySpace {
        &self.vm_space
    }

    pub fn is_child_process(&self) -> bool {
        self.inner.read().is_child_process
    }
}

impl Process {
    /// Exit the process.
    pub fn exit(&self) -> ! {
        for thread in self.inner.read().threads.iter() {
            let cpu = if let ThreadState::RunningOn(cpu) = thread.thread_state() {
                Some(cpu)
            } else {
                None
            };

            thread.set_thread_state(ThreadState::Dead);
            remove_thread(thread.thread_id());

            if let Some(cpu) = cpu
                && cpu != Cpu::current()
            {
                change_context_save_action(ContextSaveAction::Yield);
                cpu.trigger_save_context();
            }
        }

        self.inner.write().threads.clear();
        change_context_save_action(ContextSaveAction::Yield);
        Cpu::current().trigger_save_context();
        unreachable!()
    }

    /// Kill the process.
    pub fn kill(&self) {
        let threads = self.inner.read().threads.clone();
        for thread in threads {
            thread.kill();
        }
        self.inner.write().threads.clear();
    }
}

/// Builder of a process.
#[derive(Default)]
pub struct ProcessBuilder {
    vm_space: Option<VirtualMemorySpace>,
}

impl ProcessBuilder {
    /// Set the virtual memory space for the process.
    pub fn vm_space(mut self, vm_space: VirtualMemorySpace) -> Self {
        self.vm_space = Some(vm_space);
        self
    }
}

static NEXT_PROCESS_ID: AtomicUsize = AtomicUsize::new(1);

impl ProcessBuilder {
    /// Build the process.
    /// You have to contain the process yourself, otherwise it will be dropped.
    pub fn build(self) -> Result<Arc<Process>, ZodiacError> {
        Ok(Arc::new(Process {
            process_id: NEXT_PROCESS_ID.fetch_add(1, Ordering::SeqCst),
            vm_space: self.vm_space.ok_or(ZodiacError::ArgumentsNotEnough)?,
            inner: RwLock::new(ProcessInner {
                threads: Vec::new(),
                is_child_process: false,
            }),
        }))
    }
}
