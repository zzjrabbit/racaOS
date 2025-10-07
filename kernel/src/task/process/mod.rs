use core::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use alloc::{sync::Arc, vec::Vec};
use ostd::{
    arch::cpu::context::UserContext,
    mm::VmSpace,
    task::Task,
    Error as OstdError,
};
use spin::RwLock;

use crate::{
    filesystem::File,
    task::{AsThread, UserThreadData, process::user_stack::UserStack, spawn_user_thread},
};
use loader::BinaryLoader;
pub use memory::{MemoryInfo, MemoryRegion};

mod loader;
mod memory;
mod user_stack;

static PROCESSES: RwLock<Vec<Arc<Process>>> = RwLock::new(Vec::new());

pub struct Process {
    threads: RwLock<Vec<Arc<Task>>>,
    is_child_process: AtomicBool,
    exit_code: AtomicI32,
    default_files: [Arc<File>; 3],
}

impl Process {
    pub fn new(
        binary: &[u8],
        stdin: Arc<File>,
        stdout: Arc<File>,
        stderr: Arc<File>,
    ) -> Result<Arc<Self>, OstdError> {
        let new_self = Arc::new(Self {
            threads: RwLock::new(Vec::new()),
            is_child_process: AtomicBool::new(false),
            exit_code: AtomicI32::new(0),
            default_files: [stdin, stdout, stderr],
        });

        PROCESSES.write().push(new_self.clone());

        let vm_space = Arc::new(VmSpace::new());
        let memory_info = Arc::new(MemoryInfo::new(vm_space));

        let (entry, aux_vec) = memory_info.vm_space().load(binary)?;

        let mut user_stack = UserStack::new(memory_info.as_ref());

        let envp = user_stack.push(0u64);

        let argv = user_stack.push_a_lot(b"hello\0");
        user_stack.push_zero_until_aligned(16);

        user_stack.push_a_lot(&aux_vec.as_slice());

        user_stack.push(envp);
        user_stack.push(argv);
        user_stack.push(1usize);

        let user_stack_end = user_stack.stack_pointer();

        let mut user_context = UserContext::default();
        user_context.set_rip(entry);
        user_context.set_rsp(user_stack_end);

        let thread = spawn_user_thread(&new_self, user_context, memory_info);
        new_self.add_thread(thread);

        Ok(new_self)
    }

    pub fn current() -> Arc<Self> {
        let current_thread = Task::current().unwrap();
        current_thread
            .as_thread()
            .unwrap()
            .data()
            .downcast_ref::<UserThreadData>()
            .unwrap()
            .process
            .upgrade()
            .unwrap()
    }
}

impl Process {
    pub fn default_stdin(&self) -> Arc<File> {
        self.default_files[0].clone()
    }

    pub fn default_stdout(&self) -> Arc<File> {
        self.default_files[1].clone()
    }

    pub fn default_stderr(&self) -> Arc<File> {
        self.default_files[2].clone()
    }
}

impl Process {
    pub fn is_child_process(&self) -> bool {
        self.is_child_process.load(Ordering::SeqCst)
    }
}

impl Process {
    pub fn add_thread(&self, thread: Arc<Task>) {
        self.threads.write().push(thread);
    }

    pub fn remove_thread(&self, tid: usize) {
        self.threads.write().retain(|t| {
            let t_data = t.direct_downcast::<UserThreadData>().unwrap();
            t_data.tid() != tid
        });
    }
}

#[allow(dead_code)]
impl Process {
    pub fn exit(&self, exit_code: i32) {
        self.exit_code.store(exit_code, Ordering::SeqCst);
        let threads = self.threads.read().clone();
        for thread in threads.iter() {
            thread.as_thread().unwrap().exit();
        }
    }

    pub fn kill(&self) {
        self.exit_code.store(-1, Ordering::SeqCst);
        let threads = self.threads.read().clone();
        for thread in threads.iter() {
            thread.as_thread().unwrap().on_kill();
        }
    }

    pub fn exit_code(&self) -> Option<i32> {
        if self.threads.read().is_empty() {
            Some(self.exit_code.load(Ordering::SeqCst))
        } else {
            None
        }
    }
}
