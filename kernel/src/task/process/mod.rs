use core::sync::atomic::{AtomicI32, AtomicUsize, Ordering};

use alloc::{sync::{Arc, Weak}, vec::Vec};
use ostd::{arch::cpu::context::UserContext, task::Task, sync::RwLock, Error as OstdError};

use crate::{
    filesystem::File,
    mem::Vmar,
    task::{process::user_stack::UserStack, spawn_user_thread, AsThread, UserThreadData},
};
pub use memory::MemoryInfo;

mod loader;
mod memory;
mod user_stack;

static PROCESSES: RwLock<Vec<Arc<Process>>> = RwLock::new(Vec::new());

pub struct Process {
    threads: RwLock<Vec<Arc<Task>>>,
    parent: Option<Weak<Self>>,
    children: RwLock<Vec<Arc<Self>>>,
    exit_code: AtomicI32,
    default_files: [Arc<File>; 3],
    id: usize,
}

static NEXT_PROCESS_ID: AtomicUsize = AtomicUsize::new(0);

impl Process {
    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        let new_self = Arc::new(Self {
            threads: RwLock::new(Vec::new()),
            parent: Some(Arc::downgrade(self)),
            children: RwLock::new(Vec::new()),
            exit_code: AtomicI32::new(0),
            default_files: self.default_files.clone(),
            id: NEXT_PROCESS_ID.fetch_add(1, Ordering::Relaxed),
        });
        
        PROCESSES.write().push(new_self.clone());
        self.children.write().push(new_self.clone());
        new_self
    }
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
            parent: None,
            children: RwLock::new(Vec::new()),
            exit_code: AtomicI32::new(0),
            default_files: [stdin, stdout, stderr],
            id: NEXT_PROCESS_ID.fetch_add(1, Ordering::Relaxed),
        });

        PROCESSES.write().push(new_self.clone());

        let vmar = Vmar::new();
        let memory_info = Arc::new(MemoryInfo::new(vmar));

        let (entry, aux_vec) = memory_info.vmar().load(binary)?;

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

        spawn_user_thread(&new_self, user_context, memory_info, None);

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

#[allow(dead_code)]
impl Process {
    pub fn parent(&self) -> Option<Arc<Self>> {
        self.parent.clone().and_then(|parent| parent.upgrade())
    }
    
    pub fn children(&self) -> Vec<Arc<Self>> {
        self.children.read().clone()
    }

    pub fn id(&self) -> usize {
        self.id
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
        log::info!("Process {} exited with code {}", self.id(), exit_code);
        
        self.exit_code.store(exit_code, Ordering::SeqCst);
        let threads = self.threads.read().clone();
        for thread in threads.iter() {
            thread.as_thread().unwrap().exit();
        }
        
        for child in self.children.read().clone() {
            child.kill();
        }
        
        if let Some(parent) = self.parent() {
            parent.children.write().retain(|child| child.id() != self.id());
        }
    }

    pub fn kill(&self) {
        log::info!("Process {} killed", self.id());
        
        self.exit_code.store(-1, Ordering::SeqCst);
        let threads = self.threads.read().clone();
        for thread in threads.iter() {
            thread.as_thread().unwrap().on_kill();
        }
        
        for child in self.children.read().clone() {
            child.kill();
        }
        
        if let Some(parent) = self.parent() {
            parent.children.write().retain(|child| child.id() != self.id());
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
