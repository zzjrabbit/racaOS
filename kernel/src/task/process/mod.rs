use core::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use alloc::{sync::Arc, vec::Vec};
use ostd::{
    arch::cpu::context::UserContext,
    mm::{CachePolicy, FrameAllocOptions, PageFlags, PageProperty, VmSpace, PAGE_SIZE},
    task::{disable_preempt, Task},
    Error as OstdError,
};
use spin::RwLock;

use crate::{
    filesystem::File,
    mem::VmReadWrite,
    task::{spawn_user_thread, AsThread, UserThreadData},
};
use loader::BinaryLoader;
pub use memory::{MemoryInfo, MemoryRegion};

mod loader;
mod memory;

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

        let entry = memory_info.vm_space().load(binary)?;

        let stack_region = memory_info.allocate(USER_STACK_SIZE)?;
        const USER_STACK_SIZE: usize = 8 * 1024 * 1024;

        let user_stack_end = stack_region.end_address();

        let disable_preempt_guard = disable_preempt();

        let vm_space = memory_info.vm_space();

        let mut cursor = vm_space
            .cursor_mut(
                &disable_preempt_guard,
                &(stack_region.start_address()..stack_region.end_address()),
            )
            .unwrap();

        for _ in 0..USER_STACK_SIZE / PAGE_SIZE {
            let frame = FrameAllocOptions::new().alloc_frame().unwrap();

            let property = PageProperty::new_user(PageFlags::RW, CachePolicy::Writeback);
            cursor.map(frame.into(), property);
        }
        drop(cursor);
        drop(disable_preempt_guard);

        let envp = user_stack_end - size_of::<usize>();
        vm_space.write_val(envp, &0usize).unwrap();

        let path = c"hello";
        let argv = envp - path.count_bytes() - 1;
        for (id, byte) in path.to_bytes_with_nul().iter().enumerate() {
            vm_space.write_val(argv + id, byte).unwrap();
        }

        let aligned_argv = argv - 2;

        let auxv_ptr = aligned_argv - 2 * size_of::<usize>();
        vm_space.write_val(auxv_ptr, &0usize).unwrap();
        vm_space
            .write_val(auxv_ptr + size_of::<usize>(), &0usize)
            .unwrap();

        // write envp
        let envp_ptr = auxv_ptr - size_of::<usize>();
        vm_space.write_val(envp_ptr, &envp).unwrap();

        // write argv
        let argv_ptr = envp_ptr - size_of::<usize>();
        vm_space.write_val(argv_ptr, &argv).unwrap();

        // write argc
        let argc_ptr = argv_ptr - size_of::<usize>();
        vm_space.write_val(argc_ptr, &1usize).unwrap();

        let user_stack_end = argc_ptr;

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
            let t_data = t
                .as_thread()
                .unwrap()
                .data()
                .downcast_ref::<UserThreadData>()
                .unwrap();
            t_data.tid() != tid
        });
    }
}

#[allow(dead_code)]
impl Process {
    pub fn exit(&self, exit_code: i32) {
        self.exit_code.store(exit_code, Ordering::SeqCst);
        for thread in self.threads.read().clone().iter() {
            thread.as_thread().unwrap().exit();
        }
    }

    pub fn kill(&self) {
        self.exit_code.store(-1, Ordering::SeqCst);
        for thread in self.threads.read().iter() {
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
