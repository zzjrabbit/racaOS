use core::sync::atomic::{AtomicUsize, Ordering};

use ::filesystem::{FileType, Path};
use alloc::{
    boxed::Box,
    sync::{Arc, Weak},
    vec::Vec,
};
use events::Observer;
use ostd::{
    arch::cpu::context::{CpuException, UserContext},
    mm::Vaddr,
    sync::{RwLock, SpinLock, Waker},
    task::{Task, TaskOptions},
    user::{ReturnReason, UserMode},
};

use crate::{Signal, SignalEvent, SignalEventFilter, SignalKind, SignalMask, SignalQueue};

use {
    crate::syscall::syscall_handler,
    crate::trap::user_page_fault_handler,
    crate::{AsThread, MemoryInfo, Process, Thread, thread::CallBacks},
    ::filesystem::File,
    memory::Vmar,
};

mod filesystem;
mod fs_resolver;

pub use filesystem::*;
pub use fs_resolver::*;

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

                let data = current.direct_downcast::<UserThreadData>().unwrap();
                data.memory_info().vmar().activate();
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
    memory_info: RwLock<Arc<MemoryInfo>>,
    fs_info: Arc<FileSystemInfo>,
    fs_resolver: RwLock<FsResolver>,
    signal_mask: RwLock<SignalMask>,
    signal_queues: SignalQueue,
    signalled_waker: SpinLock<Option<Arc<Waker>>>,
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
            memory_info: RwLock::new(memory_info),
            tid_address: RwLock::new(None),
            tid: TID.fetch_add(1, Ordering::SeqCst),
            fs_info: Arc::new(FileSystemInfo::new(stdin, stdout, stderr)),
            fs_resolver: RwLock::new(FsResolver::new(Path::from("/"), Path::from("/"))),
            signal_mask: RwLock::new(SignalMask::default()),
            signal_queues: SignalQueue::new(),
            signalled_waker: SpinLock::new(None),
        }
    }

    pub fn new_all(
        process: &Arc<Process>,
        memory_info: Arc<MemoryInfo>,
        fs_info: Arc<FileSystemInfo>,
        fs_resolver: FsResolver,
        tid_address: Option<Vaddr>,
    ) -> Self {
        Self {
            process: Arc::downgrade(process),
            memory_info: RwLock::new(memory_info),
            tid_address: RwLock::new(tid_address),
            tid: TID.fetch_add(1, Ordering::SeqCst),
            fs_info,
            fs_resolver: RwLock::new(fs_resolver),
            signal_mask: RwLock::new(SignalMask::default()),
            signal_queues: SignalQueue::new(),
            signalled_waker: SpinLock::new(None),
        }
    }
}

impl UserThreadData {
    pub fn tid(&self) -> usize {
        self.tid
    }
}

impl UserThreadData {
    pub fn memory_info(&self) -> Arc<MemoryInfo> {
        self.memory_info.read().clone()
    }

    pub fn replace_memory_info(&self, memory_info: Arc<MemoryInfo>) {
        *self.memory_info.write() = memory_info;
    }

    pub fn fs_info(&self) -> &Arc<FileSystemInfo> {
        &self.fs_info
    }
}

impl UserThreadData {
    pub fn block_signal(&self, signal: Signal) {
        *self.signal_mask.write() += signal;
    }

    pub fn unblock_signal(&self, signal: Signal) {
        *self.signal_mask.write() -= signal;
    }

    pub fn blocked_signals(&self) -> SignalMask {
        self.signal_mask.read().clone()
    }

    pub fn has_pending_signals(&self) -> bool {
        self.signal_queues.has_pending(self.blocked_signals())
    }

    pub fn set_signalled_waker(&self, waker: Arc<Waker>) {
        *self.signalled_waker.lock() = Some(waker);
    }

    pub fn clear_signalled_waker(&self) {
        *self.signalled_waker.lock() = None;
    }

    pub fn wake_signalled_waker(&self) {
        if let Some(waker) = &*self.signalled_waker.lock() {
            waker.wake_up();
        }
    }

    pub fn enqueue_signal(&self, signal_kind: SignalKind) {
        let process = self.process.upgrade().unwrap();

        let signal_disposition = process.signal_disposition();
        let signal_disposition = signal_disposition.lock();

        let signal = signal_kind.signal();
        if signal_disposition.get(signal).will_ignore(signal) {
            return;
        }

        self.enqueue_signal_locked(signal_kind);
    }

    pub(crate) fn enqueue_signal_locked(&self, signal_kind: SignalKind) {
        self.signal_queues.enqueue(signal_kind);
        self.wake_signalled_waker();
    }

    pub fn dequeue_signal(&self, mask: SignalMask) -> Option<SignalKind> {
        self.signal_queues.dequeue(&mask)
    }

    pub fn register_signal_queue_observer(
        &self,
        observer: Weak<dyn Observer<SignalEvent>>,
        filter: SignalEventFilter,
    ) {
        self.signal_queues.register_observer(observer, filter);
    }

    pub fn unregister_signal_queue_observer(&self, observer: &Weak<dyn Observer<SignalEvent>>) {
        self.signal_queues.unregister_observer(observer);
    }
}

impl UserThreadData {
    pub fn cwd(&self) -> Path {
        self.fs_resolver.read().cwd().clone()
    }

    pub fn root(&self) -> Path {
        self.fs_resolver.read().root().clone()
    }

    pub fn set_cwd(&self, path: Path) {
        self.fs_resolver.write().set_cwd(path);
    }

    pub fn set_root(&self, path: Path) {
        self.fs_resolver.write().set_root(path);
    }

    pub fn open_file(&self, path: &Path) -> Option<Arc<File>> {
        self.fs_resolver.read().open_file(path)
    }

    pub fn create_file(&self, path: &Path, file_type: FileType) -> Option<Arc<File>> {
        self.fs_resolver.write().create_file(path, file_type)
    }

    pub fn clone_fs_resolver(&self) -> FsResolver {
        self.fs_resolver.read().clone()
    }
}
