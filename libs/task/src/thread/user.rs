use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{
    boxed::Box,
    sync::{Arc, Weak},
    vec::Vec,
};
use ostd::{
    arch::cpu::context::{CpuException, UserContext},
    mm::Vaddr,
    sync::RwLock,
    task::{Task, TaskOptions},
    user::{ReturnReason, UserMode},
};

use {
    crate::syscall::syscall_handler,
    crate::trap::user_page_fault_handler,
    crate::{AsThread, MemoryInfo, Process, Thread, thread::CallBacks},
    ::filesystem::File,
    memory::Vmar,
};

mod filesystem;

pub use filesystem::*;

static THREADS: RwLock<Vec<Arc<Task>>> = RwLock::new(Vec::new());

struct UserThreadCallbacks {
    process: Weak<Process>,
    tid: usize,
    vmar: Arc<Vmar>,
}

impl CallBacks for UserThreadCallbacks {
    fn on_exit(&self) {
        self.process.upgrade().unwrap().remove_thread(self.tid);
    }

    fn on_kill(&self) {
        self.process.upgrade().unwrap().remove_thread(self.tid);
    }

    fn pre_execute(&self) {
        self.vmar.activate();
    }
}

pub fn spawn_user_thread(
    process: &Arc<Process>,
    user_context: UserContext,
    memory_info: Arc<MemoryInfo>,
    user_data: Option<UserThreadData>,
) -> Arc<Task> {
    let user_entry = || {
        {
            let current = Task::current().unwrap();
            let data = current.direct_downcast::<UserThreadData>().unwrap();
            data.memory_info().vmar().activate();
        }

        let mut user_mode = UserMode::new(user_context);
        user_mode.context().activate_tls_pointer();
        
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
                        _ => {
                            let process = Process::current();

                            log::error!("Unhandled exception: {:?}", exception);
                            log::error!("Context: {:#x?}", context);
                            log::error!("Process: {}", process.id());

                            process.exit(-1);
                        }
                    }
                }
                _ => panic!("Thread error: {:x?}", return_reason),
            }

            Task::yield_now();
        }
    };

    let thread = Arc::new_cyclic(|weak_task| {
        let thread_data = Box::new(user_data.unwrap_or(UserThreadData::new(
            process.default_stdin(),
            process.default_stdout(),
            process.default_stderr(),
            process,
            memory_info,
        )));

        let tid = thread_data.tid();
        let vmar = thread_data.memory_info().vmar();

        let thread = Thread::new(
            weak_task.clone(),
            thread_data,
            Box::new(UserThreadCallbacks {
                process: Arc::downgrade(process),
                tid,
                vmar,
            }),
        );

        TaskOptions::new(user_entry)
            .data(Arc::new(thread))
            .build()
            .unwrap()
    });

    THREADS.write().push(thread.clone());
    process.add_thread(thread.clone());
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

static TID: AtomicUsize = AtomicUsize::new(0);

impl UserThreadData {
    pub fn new(
        stdin: Arc<File>,
        stdout: Arc<File>,
        stderr: Arc<File>,
        process: &Arc<Process>,
        memory_info: Arc<MemoryInfo>,
    ) -> Self {
        Self {
            process: Arc::downgrade(process),
            memory_info,
            tid_address: RwLock::new(None),
            tid: TID.fetch_add(1, Ordering::SeqCst),
            fs_info: Arc::new(FileSystemInfo::new(stdin, stdout, stderr)),
        }
    }

    pub fn new_all(
        process: &Arc<Process>,
        memory_info: Arc<MemoryInfo>,
        fs_info: Arc<FileSystemInfo>,
        tid_address: Option<Vaddr>,
    ) -> Self {
        Self {
            process: Arc::downgrade(process),
            memory_info,
            tid_address: RwLock::new(tid_address),
            tid: TID.fetch_add(1, Ordering::SeqCst),
            fs_info,
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
