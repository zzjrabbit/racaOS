mod fifo;

use core::sync::atomic::{AtomicBool, Ordering};

use alloc::sync::{Arc, Weak};
pub use fifo::*;
use spin::Once;

use crate::{
    hal::{context::TrapFrame, cpu_num, enable_interrupts, trap::set_kernel_stack},
    task::{Process, Thread, ThreadBuilder, ThreadId},
};

pub trait Scheduler<T = Arc<Thread>>: Sync + Send {
    fn with_local_queue(&self, f: &mut dyn FnMut(&dyn LocalQueue));
    fn with_local_queue_mut(&self, f: &mut dyn FnMut(&mut dyn LocalQueue));
    /// Retain the threads that satisfy the predicate.
    fn retain(&self, f: &dyn Fn(&T) -> bool);
}

pub trait LocalQueue<T = Arc<Thread>, W = Weak<Thread>> {
    /// Returns the current thread.
    /// Note that the current thread is not in the ready queue.
    fn current(&self) -> Option<W>;
    /// Push a thread into the local queue.
    fn enqueue(&mut self, thread: T);
    /// Pick the next thread to execute, and deque the thread.
    fn deque_next_thread(&mut self) -> Option<T>;
}

pub fn set_scheduler(scheduler: &'static dyn Scheduler) {
    SCHEDULER.set_scheduler(scheduler);

    for _ in 0..cpu_num() {
        let thread = ThreadBuilder::default()
            .entry(idle)
            .kernel_mode()
            .process(Process::kernel())
            .build()
            .unwrap();
        thread.spawn();
    }
}

fn idle() -> ! {
    loop {
        core::hint::spin_loop();
    }
}

pub fn start_schedule() {
    START_SCHEDULE.store(true, Ordering::SeqCst);
    enable_interrupts();
}

pub(crate) fn ap_init() {
    while !START_SCHEDULE.load(Ordering::SeqCst) {
        core::hint::spin_loop();
    }
    log::info!("AP interrupts enabled.");
    enable_interrupts();
}

static START_SCHEDULE: AtomicBool = AtomicBool::new(false);

static SCHEDULER: SchedulerWrapper = SchedulerWrapper::new();

struct SchedulerWrapper {
    scheduler: Once<&'static dyn Scheduler>,
}

impl SchedulerWrapper {
    const fn new() -> Self {
        Self {
            scheduler: Once::new(),
        }
    }
}

impl SchedulerWrapper {
    fn set_scheduler(&self, scheduler: &'static dyn Scheduler) {
        self.scheduler.call_once(|| scheduler);
    }
}

impl SchedulerWrapper {
    fn add(&self, thread: Arc<Thread>) {
        self.scheduler
            .get()
            .unwrap()
            .with_local_queue_mut(&mut |queue| queue.enqueue(thread.clone()));
    }

    fn remove(&self, thread_id: ThreadId) {
        self.scheduler
            .get()
            .unwrap()
            .retain(&|t| t.thread_id() != thread_id);
    }

    fn schedule(&self, context: &mut TrapFrame) {
        self.scheduler
            .get()
            .unwrap()
            .with_local_queue_mut(&mut |queue| {
                if let Some(current) = queue.current()
                    && let Some(current) = current.upgrade()
                {
                    current.set_context(context.clone());
                    if !current.thread_state().is_blocked() {
                        current.ready();
                        queue.enqueue(current.clone());
                    }
                }

                let next = queue.deque_next_thread().expect("CPU Hungry.");
                next.run();
                *context = next.context();

                set_kernel_stack(next.kernel_stack());

                #[cfg(target_arch = "x86_64")]
                {
                    use x86_64::{
                        VirtAddr,
                        registers::model_specific::{FsBase, GsBase},
                    };
                    if let Some(fs_base) = next.fs_base() {
                        FsBase::write(VirtAddr::new(fs_base as u64));
                    }
                    if let Some(gs_base) = next.gs_base() {
                        GsBase::write(VirtAddr::new(gs_base as u64));
                    }
                }

                next.process()
                    .expect("No process contains the next thread.")
                    .vm_space()
                    .switch();
            });
    }

    fn current(&self) -> Arc<Thread> {
        let mut current = None;
        self.scheduler
            .get()
            .unwrap()
            .with_local_queue(&mut |queue| {
                current = queue.current().unwrap().upgrade();
            });
        current.unwrap()
    }
}

mod wrappers {
    use super::*;

    pub fn add_thread(thread: Arc<Thread>) {
        SCHEDULER.add(thread);
    }

    pub fn remove_thread(thread_id: ThreadId) {
        SCHEDULER.remove(thread_id);
    }

    pub fn schedule(context: &mut TrapFrame) {
        SCHEDULER.schedule(context);
    }

    pub fn current_thread() -> Arc<Thread> {
        SCHEDULER.current()
    }
}

pub(crate) use wrappers::*;
