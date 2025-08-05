use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{sync::Arc, vec::Vec};
use spin::{Lazy, RwLock};

use crate::{
    ZodiacError,
    hal::cpu::Cpu,
    mem::VirtualMemorySpace,
    task::{Thread, ThreadId, ThreadState, remove_thread},
};

pub type ProcessId = usize;

pub struct Process {
    process_id: ProcessId,
    vm_space: VirtualMemorySpace,
    inner: RwLock<ProcessInner>,
}

struct ProcessInner {
    threads: Vec<Arc<Thread>>,
}

impl Process {
    pub(crate) fn new_kernel() -> Arc<Self> {
        Arc::new(Self {
            process_id: 0,
            vm_space: VirtualMemorySpace::new_user(),
            inner: RwLock::new(ProcessInner {
                threads: Vec::new(),
            }),
        })
    }

    pub fn kernel() -> Arc<Self> {
        static KERNEL: Lazy<Arc<Process>> = Lazy::new(Process::new_kernel);
        KERNEL.clone()
    }
}

impl Process {
    pub fn process_id(&self) -> ProcessId {
        self.process_id
    }

    pub fn add_thread(&self, thread: Arc<Thread>) {
        self.inner.write().threads.push(thread);
    }

    pub fn remove_thread(&self, thread_id: ThreadId) {
        self.inner
            .write()
            .threads
            .retain(|thread| thread.thread_id() != thread_id);
    }

    pub fn vm_space(&self) -> &VirtualMemorySpace {
        &self.vm_space
    }
}

impl Process {
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
                cpu.trigger_schedule();
            }
        }
        
        self.inner.write().threads.clear();
        Cpu::current().trigger_schedule();
        unreachable!()
    }

    pub fn kill(&self) {
        let threads = self.inner.read().threads.clone();
        for thread in threads {
            thread.kill();
        }
    }
}

#[derive(Default)]
pub struct ProcessBuilder {
    vm_space: Option<VirtualMemorySpace>,
}

impl ProcessBuilder {
    pub fn vm_space(mut self, vm_space: VirtualMemorySpace) -> Self {
        self.vm_space = Some(vm_space);
        self
    }
}

impl ProcessBuilder {
    pub fn build(self) -> Result<Arc<Process>, ZodiacError> {
        static NEXT_PROCESS_ID: AtomicUsize = AtomicUsize::new(1);

        Ok(Arc::new(Process {
            process_id: NEXT_PROCESS_ID.fetch_add(1, Ordering::SeqCst),
            vm_space: self.vm_space.ok_or(ZodiacError::ArgumentsNotEnough)?,
            inner: RwLock::new(ProcessInner {
                threads: Vec::new(),
            }),
        }))
    }
}
