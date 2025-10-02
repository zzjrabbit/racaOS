use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};
use ostd::{
    mm::Vaddr,
    task::{Task, TaskOptions},
};
use spin::{Once, RwLock};

use crate::{
    filesystem::{AccessMode, File, OpenFlags},
    task::Process,
};

mod memory;
mod filesystem;

pub use memory::*;
pub use filesystem::*;

type FileDescriptorInfo = (u64, AccessMode, OpenFlags, Arc<File>);

static THREADS: RwLock<Vec<Arc<Task>>> = RwLock::new(Vec::new());

pub fn create_kernel_thread(entry: fn()) -> Arc<Task> {
    let task = Arc::new(TaskOptions::new(entry).build().unwrap());
    task.run();
    THREADS.write().push(task.clone());
    task
}

pub struct ThreadData {
    pub process: Weak<Process>,
    pub tid_address: RwLock<Option<Vaddr>>,
    pub entry: Once<usize>,
    pub stack: Once<usize>,
    tid: usize,
    memory_info: MemoryInfo,
    fs_info: FileSystemInfo,
    dead: AtomicBool,
}

impl ThreadData {
    pub fn new(
        stdin: Arc<File>,
        stdout: Arc<File>,
        stderr: Arc<File>,
        process: &Arc<Process>,
    ) -> Self {
        static TID: AtomicUsize = AtomicUsize::new(0);

        Self {
            process: Arc::downgrade(process),
            memory_info: MemoryInfo::new(),
            tid_address: RwLock::new(None),
            entry: Once::new(),
            stack: Once::new(),
            tid: TID.fetch_add(1, Ordering::SeqCst),
            fs_info: FileSystemInfo::new(stdin, stdout, stderr),
            dead: AtomicBool::new(false),
        }
    }
}

impl ThreadData {
    pub fn tid(&self) -> usize {
        self.tid
    }
}

impl ThreadData {
    pub fn memory_info(&self) -> &MemoryInfo {
        &self.memory_info
    }

    pub fn fs_info(&self) -> &FileSystemInfo {
        &self.fs_info
    }
}

impl ThreadData {
    pub fn on_exit(&self) {
        self.dead.store(true, Ordering::SeqCst);
        
        let process = self.process.upgrade().unwrap();
        process.remove_thread(self.tid());
    }
    
    pub fn on_kill(&self) {
        self.dead.store(true, Ordering::SeqCst);
        
        let process = self.process.upgrade().unwrap();
        process.remove_thread(self.tid());
    }
    
    pub fn is_dead(&self) -> bool {
        self.dead.load(Ordering::SeqCst)
    }
}
