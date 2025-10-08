use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{
    boxed::Box,
    sync::{Arc, Weak},
    vec::Vec,
};
use ostd::{
    arch::cpu::context::{CpuException, UserContext},
    mm::{Vaddr, VmSpace},
    sync::RwLock,
    task::{Task, TaskOptions},
    user::{ReturnReason, UserMode},
};

use crate::{
    filesystem::File,
    syscall::syscall_handler,
    task::{thread::CallBacks, AsThread, MemoryInfo, Process, Thread},
    trap::user_page_fault_handler,
};

mod filesystem;

pub use filesystem::*;

static THREADS: RwLock<Vec<Arc<Task>>> = RwLock::new(Vec::new());

struct UserThreadCallbacks {
    process: Weak<Process>,
    tid: usize,
    vm_space: Arc<VmSpace>,
}

impl CallBacks for UserThreadCallbacks {
    fn on_exit(&self) {
        self.process.upgrade().unwrap().remove_thread(self.tid);
    }

    fn on_kill(&self) {
        self.process.upgrade().unwrap().remove_thread(self.tid);
    }

    fn pre_execute(&self) {
        self.vm_space.activate();
    }
}

pub fn spawn_user_thread(
    process: &Arc<Process>,
    user_context: UserContext,
    memory_info: Arc<MemoryInfo>,
) -> Arc<Task> {
    let user_entry = || {
        let mut user_mode = {
            let current = Task::current().unwrap();
            let data = current.direct_downcast::<UserThreadData>().unwrap();

            data.memory_info().vm_space().activate();
            UserMode::new(user_context)
        };

        loop {
            {
                let current = Task::current().unwrap();
                let data = current.as_thread().unwrap();

                if data.is_dead() {
                    break;
                }
            }

            let return_reason = user_mode.execute(|| false);

            match return_reason {
                ReturnReason::UserSyscall => {
                    syscall_handler(user_mode.context_mut());
                }
                ReturnReason::UserException => {
                    let context = user_mode.context_mut();
                    let exception = context.take_exception().unwrap();

                    match exception {
                        CpuException::PageFault(_) => user_page_fault_handler(&exception).unwrap(),
                        _ => panic!("Unhandled exception: {:?}", exception),
                    }
                }
                _ => panic!("Thread error: {:x?}", return_reason),
            }

            Task::yield_now();
        }
    };

    let thread = Arc::new_cyclic(|weak_task| {
        let thread_data = Box::new(UserThreadData::new(
            process.default_stdin(),
            process.default_stdout(),
            process.default_stderr(),
            process,
            memory_info,
        ));

        let tid = thread_data.tid();
        let vm_space = thread_data.memory_info().vm_space();

        let thread = Thread::new(
            weak_task.clone(),
            thread_data,
            Box::new(UserThreadCallbacks {
                process: Arc::downgrade(process),
                tid,
                vm_space,
            }),
        );

        TaskOptions::new(user_entry)
            .data(Arc::new(thread))
            .build()
            .unwrap()
    });

    THREADS.write().push(thread.clone());
    thread.run();

    thread
}

pub struct UserThreadData {
    pub process: Weak<Process>,
    pub tid_address: RwLock<Option<Vaddr>>,
    tid: usize,
    memory_info: Arc<MemoryInfo>,
    fs_info: Arc<FileSystemInfo>,
}

impl UserThreadData {
    pub fn new(
        stdin: Arc<File>,
        stdout: Arc<File>,
        stderr: Arc<File>,
        process: &Arc<Process>,
        memory_info: Arc<MemoryInfo>,
    ) -> Self {
        static TID: AtomicUsize = AtomicUsize::new(0);

        Self {
            process: Arc::downgrade(process),
            memory_info,
            tid_address: RwLock::new(None),
            tid: TID.fetch_add(1, Ordering::SeqCst),
            fs_info: Arc::new(FileSystemInfo::new(stdin, stdout, stderr)),
        }
    }
}

impl UserThreadData {
    pub fn tid(&self) -> usize {
        self.tid
    }
}

impl UserThreadData {
    pub fn memory_info(&self) -> &Arc<MemoryInfo> {
        &self.memory_info
    }

    pub fn fs_info(&self) -> &Arc<FileSystemInfo> {
        &self.fs_info
    }
}
