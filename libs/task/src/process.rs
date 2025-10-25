use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};
use errors::Result;
use ostd::{
    arch::cpu::context::UserContext,
    sync::{Mutex, RwLock, Waker},
    task::Task,
};

use crate::{Signal, SignalDisposition, SignalKind};

pub use memory::MemoryInfo;
pub use status::ProcessStatus;
pub(crate) use user_stack::UserStack;
use {
    crate::{AsThread, UserThreadData, spawn_user_thread},
    ::memory::Vmar,
    filesystem::File,
};

mod loader;
mod memory;
mod status;
mod user_stack;

static PROCESSES: RwLock<Vec<Arc<Process>>> = RwLock::new(Vec::new());

pub struct Process {
    threads: RwLock<Vec<Arc<Task>>>,
    parent: Option<Weak<Self>>,
    children: RwLock<Vec<Arc<Self>>>,
    default_files: [Arc<File>; 3],
    id: usize,
    status: Mutex<ProcessStatus>,

    child_death_signal: Mutex<Signal>,
    signal_disposition: Arc<Mutex<SignalDisposition>>,
    wakers: RwLock<Vec<Arc<Waker>>>,
}

static NEXT_PROCESS_ID: AtomicUsize = AtomicUsize::new(1);

impl Process {
    pub fn fork(
        self: &Arc<Self>,
        child_death_signal: Signal,
        signal_disposition: Arc<Mutex<SignalDisposition>>,
    ) -> Arc<Self> {
        let new_self = Arc::new(Self {
            threads: RwLock::new(Vec::new()),
            parent: Some(Arc::downgrade(self)),
            children: RwLock::new(Vec::new()),
            status: Mutex::new(ProcessStatus::Alive),
            default_files: self.default_files.clone(),
            id: NEXT_PROCESS_ID.fetch_add(1, Ordering::Relaxed),
            child_death_signal: Mutex::new(child_death_signal),
            wakers: RwLock::new(Vec::new()),
            signal_disposition,
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
    ) -> Result<Arc<Self>> {
        let new_self = Arc::new(Self {
            threads: RwLock::new(Vec::new()),
            parent: None,
            children: RwLock::new(Vec::new()),
            status: Mutex::new(ProcessStatus::Alive),
            default_files: [stdin, stdout, stderr],
            id: NEXT_PROCESS_ID.fetch_add(1, Ordering::Relaxed),
            child_death_signal: Mutex::new(Signal::SIGCHLD),
            signal_disposition: Arc::new(Mutex::new(SignalDisposition::default())),
            wakers: RwLock::new(Vec::new()),
        });

        PROCESSES.write().push(new_self.clone());

        let vmar = Vmar::new();
        let memory_info = Arc::new(MemoryInfo::new(vmar));

        let (entry, aux_vec) = memory_info.load(0, None, binary)?;
        log::trace!("both program and dynamic linker are loaded into memory!");

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

    pub fn signal_disposition(&self) -> Arc<Mutex<SignalDisposition>> {
        self.signal_disposition.clone()
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

        *self.status.lock() = ProcessStatus::Zombie(exit_code);
        let threads = self.threads.read().clone();
        for thread in threads.iter() {
            thread.as_thread().unwrap().exit();
        }
        self.threads.write().clear();

        /*for child in self.children.read().clone() {
            child.kill();
        }*/

        if let Some(parent) = self.parent() {
            {
                let child_death_signal = parent.child_death_signal.lock();
                parent.enqueue_signal(SignalKind::new_kernel(*child_death_signal));
            }

            for waker in self.wakers.read().clone() {
                waker.wake_up();
            }
            self.wakers.write().clear();
        }
    }

    pub fn kill(&self) {
        log::info!("Process {} killed", self.id());

        *self.status.lock() = ProcessStatus::Zombie(-1);
        let threads = self.threads.read().clone();
        for thread in threads.iter() {
            thread.as_thread().unwrap().on_kill();
        }
        self.threads.write().clear();

        for child in self.children.read().clone() {
            child.kill();
        }

        if let Some(parent) = self.parent() {
            {
                let child_death_signal = parent.child_death_signal.lock();
                parent.enqueue_signal(SignalKind::new_kernel(*child_death_signal));
            }

            for waker in self.wakers.read().clone() {
                waker.wake_up();
            }
            self.wakers.write().clear();
        }
    }

    pub fn status(&self) -> ProcessStatus {
        *self.status.lock()
    }

    pub fn add_zombie_waker(&self, waker: Arc<Waker>) {
        self.wakers.write().push(waker);
    }

    pub(crate) fn clear_zombie(&self) {
        if self.status().is_zombie() {
            if let Some(parent) = self.parent() {
                parent
                    .children
                    .write()
                    .retain(|child| child.id() != self.id());
            }
        }
    }
}

impl Process {
    pub fn enqueue_signal(&self, signal_kind: SignalKind) {
        if self.status().is_zombie() {
            return;
        }

        let signal_disposition = self.signal_disposition.lock();

        // Drop the signal if it's ignored. See explanation at `enqueue_signal_locked`.
        let signal = signal_kind.signal();
        if signal_disposition.get(signal).will_ignore(signal) {
            return;
        }

        let threads = self.threads.read();

        // Enqueue the signal to the first thread that does not block the signal.
        for thread in threads.as_slice() {
            let data = thread.direct_downcast::<UserThreadData>().unwrap();
            if !data.blocked_signals().contains(signal) {
                data.enqueue_signal_locked(signal_kind);
            }
        }

        // If all threads block the signal, enqueue the signal to the main thread.
        let thread = threads[0].clone();
        let data = thread.direct_downcast::<UserThreadData>().unwrap();
        data.enqueue_signal_locked(signal_kind);
    }
}
