use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};
use spin::RwLock;

use crate::{
    ZodiacError,
    hal::{
        context::TrapFrame,
        cpu::Cpu,
        trap::{ContextSaveAction, change_context_save_action},
    },
    mem::VirtualAddress,
    task::{Process, add_thread, current_thread, remove_thread},
};

#[cfg(target_arch = "x86_64")]
use x86_64::{
    VirtAddr,
    registers::model_specific::{FsBase, GsBase},
};

pub type ThreadId = usize;

const KERNEL_STACK_SIZE: usize = 64 * 1024; // 64k

/// Thread structure.
/// This is basically a container of context.
pub struct Thread {
    inner: RwLock<ThreadInner>,
    thread_id: ThreadId,
    process: Weak<Process>,
    kernel_stack: Vec<u8>,
}

struct ThreadInner {
    thread_state: ThreadState,
    context: TrapFrame,
    #[cfg(target_arch = "x86_64")]
    fs_base: Option<VirtualAddress>,
    #[cfg(target_arch = "x86_64")]
    gs_base: Option<VirtualAddress>,
}

impl Thread {
    /// Get the current thread.
    pub fn current() -> Arc<Self> {
        current_thread()
    }
    
    pub fn clone_thread(self: &Arc<Self>, new_stack: VirtualAddress) {
        change_context_save_action(ContextSaveAction::Clone(self.clone(), new_stack));
        Cpu::current().trigger_save_context();
    }

    pub(crate) fn clone_impl(
        &self,
        mut context: TrapFrame,
        new_stack: VirtualAddress,
    ) -> Arc<Self> {
        context.set_stack(new_stack);
        let new_thread = Arc::new(Self {
            thread_id: NEXT_THREAD_ID.fetch_add(1, Ordering::SeqCst),
            process: self.process.clone(),
            kernel_stack: self.kernel_stack.clone(),
            inner: RwLock::new(ThreadInner {
                thread_state: self.thread_state(),
                context,
                fs_base: self.fs_base(),
                gs_base: self.gs_base(),
            }),
        });
        if let Some(process) = self.process() {
            process.add_thread(new_thread.clone());
        }
        if !new_thread.thread_state().is_blocked() {
            new_thread.spawn();
        }
        new_thread
    }

    /// Spawn this thread.
    pub fn spawn(self: &Arc<Self>) {
        add_thread(self.clone());
    }
}

impl Thread {
    /// Get the process of this thread.
    pub fn process(&self) -> Option<Arc<Process>> {
        self.process.upgrade()
    }

    /// Get the thread id of this thread.
    pub fn thread_id(&self) -> ThreadId {
        self.thread_id
    }

    /// Get the thread state of this thread.
    pub fn thread_state(&self) -> ThreadState {
        self.inner.read().thread_state
    }

    pub(crate) fn set_thread_state(&self, state: ThreadState) {
        self.inner.write().thread_state = state;
    }

    pub(crate) fn context(&self) -> TrapFrame {
        self.inner.read().context.clone()
    }

    pub(crate) fn set_context(&self, context: TrapFrame) {
        self.inner.write().context = context;
    }

    pub(crate) fn kernel_stack(&self) -> VirtualAddress {
        self.kernel_stack.as_ptr() as VirtualAddress + self.kernel_stack.len()
    }

    /// Set the FS base register for this thread.
    /// This function only exists on x86_64.
    #[cfg(target_arch = "x86_64")]
    pub fn set_fs_base(&self, fs_base: VirtualAddress) {
        self.inner.write().fs_base = Some(fs_base);
        FsBase::write(VirtAddr::new(fs_base as u64));
    }

    /// Set the GS base register for this thread.
    /// This function only exists on x86_64.
    #[cfg(target_arch = "x86_64")]
    pub fn set_gs_base(&self, gs_base: VirtualAddress) {
        self.inner.write().gs_base = Some(gs_base);
        GsBase::write(VirtAddr::new(gs_base as u64));
    }

    /// Get the FS base register for this thread.
    /// This function only exists on x86_64.
    #[cfg(target_arch = "x86_64")]
    pub fn fs_base(&self) -> Option<VirtualAddress> {
        self.inner.read().fs_base
    }

    /// Get the GS base register for this thread.
    /// This function only exists on x86_64.
    #[cfg(target_arch = "x86_64")]
    pub fn gs_base(&self) -> Option<VirtualAddress> {
        self.inner.read().gs_base
    }
}

impl Thread {
    /// Block this thread.
    pub fn block(&self) {
        self.set_thread_state(ThreadState::Blocked);
        self.r#yield();
    }

    /// Yield the CPU to another thread.
    pub fn r#yield(&self) {
        change_context_save_action(ContextSaveAction::Yield);
        Cpu::current().trigger_save_context();
    }

    pub(crate) fn run(&self) {
        self.set_thread_state(ThreadState::RunningOn(Cpu::current()));
    }

    pub(crate) fn ready(&self) {
        self.set_thread_state(ThreadState::Ready);
    }
}

impl Thread {
    /// Exit this thread.
    pub fn exit(&self) -> ! {
        remove_thread(self.thread_id());
        self.process().unwrap().remove_thread(self.thread_id());
        self.set_thread_state(ThreadState::Dead);

        self.r#yield();
        unreachable!()
    }

    /// Kill this thread.
    pub fn kill(&self) {
        remove_thread(self.thread_id());
        self.process().unwrap().remove_thread(self.thread_id());

        let cpu = if let ThreadState::RunningOn(cpu) = self.thread_state() {
            Some(cpu)
        } else {
            None
        };
        self.set_thread_state(ThreadState::Dead);

        if let Some(cpu) = cpu {
            change_context_save_action(ContextSaveAction::Yield);
            cpu.trigger_save_context();
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ThreadState {
    Ready,
    RunningOn(Cpu),
    Blocked,
    Dead,
}

impl ThreadState {
    pub fn is_blocked(&self) -> bool {
        match self {
            Self::Ready | Self::RunningOn(_) => false,
            _ => true,
        }
    }

    pub fn on_cpu(&self) -> Option<Cpu> {
        match self {
            ThreadState::RunningOn(cpu) => Some(*cpu),
            _ => None,
        }
    }
}

/// Builder of a thread.
pub struct ThreadBuilder {
    process: Option<Arc<Process>>,
    kernel_stack_size: usize,
    user_mode: bool,
    entry: Option<usize>,
    stack: Option<usize>,
}

impl Default for ThreadBuilder {
    fn default() -> Self {
        Self {
            process: None,
            kernel_stack_size: KERNEL_STACK_SIZE,
            user_mode: true,
            entry: None,
            stack: None,
        }
    }
}

impl ThreadBuilder {
    /// Set the entry point of the thread.
    /// This is a necessary option.
    pub fn entry(mut self, entry: fn() -> !) -> Self {
        self.entry = Some(entry as usize);
        self
    }

    /// Set the stack of the thread.
    /// This is a necessary option for user threads, and it does nothing for kernel threads.
    pub fn stack(mut self, stack: usize) -> Self {
        self.stack = Some(stack);
        self
    }

    /// Set the process of the thread.
    /// This is a necessary option.
    pub fn process(mut self, process: Arc<Process>) -> Self {
        self.process = Some(process);
        self
    }

    /// Set the kernel stack size of the thread.
    /// This is optional.
    pub fn kernel_stack_size(mut self, size: usize) -> Self {
        self.kernel_stack_size = size;
        self
    }

    /// Make the thread in kernel mode.
    pub fn kernel_mode(mut self) -> Self {
        self.user_mode = false;
        self
    }
}

static NEXT_THREAD_ID: AtomicUsize = AtomicUsize::new(0);

impl ThreadBuilder {
    /// Create the thread.
    pub fn build(self) -> Result<Arc<Thread>, ZodiacError> {
        let kernel_stack = alloc::vec![0; self.kernel_stack_size];

        let process = self.process.ok_or(ZodiacError::ArgumentsNotEnough)?;
        let entry = self.entry.ok_or(ZodiacError::ArgumentsNotEnough)?;
        let stack = if self.user_mode {
            self.stack.ok_or(ZodiacError::ArgumentsNotEnough)?
        } else {
            0
        };

        let mut context = TrapFrame::default();
        context.init(&kernel_stack, entry, stack, self.user_mode);

        let thread = Arc::new(Thread {
            process: Arc::downgrade(&process),
            thread_id: NEXT_THREAD_ID.fetch_add(1, Ordering::SeqCst),
            kernel_stack,
            inner: RwLock::new(ThreadInner {
                context,
                thread_state: ThreadState::Ready,
                #[cfg(target_arch = "x86_64")]
                fs_base: None,
                #[cfg(target_arch = "x86_64")]
                gs_base: None,
            }),
        });

        process.add_thread(thread.clone());
        Ok(thread)
    }
}
