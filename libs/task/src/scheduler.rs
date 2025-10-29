use alloc::{boxed::Box, collections::vec_deque::VecDeque, sync::Arc, vec::Vec};
use ostd::{
    cpu::{CpuId, PinCurrentCpu, num_cpus},
    sync::SpinLock,
    task::{
        Task, disable_preempt, inject_post_schedule_handler, inject_pre_schedule_handler,
        scheduler::{
            EnqueueFlags, LocalRunQueue, Scheduler, UpdateFlags, info::CommonSchedInfo,
            inject_scheduler,
        },
    },
};

use crate::{AsThread, AsThreadLocal, UserThreadData};

fn pre_schedule_handler() {
    let Some(task) = Task::current() else {
        return;
    };
    if let Some(thread_local) = task.as_thread_local() {
        thread_local.fpu().before_schedule();
    }
}

fn post_schedule_handler() {
    let task = Task::current().unwrap();
    if let Some(data) = task.direct_downcast::<UserThreadData>() {
        data.memory_info().vmar().activate();
    }
    if let Some(thread_local) = task.as_thread_local() {
        thread_local.fpu().after_schedule();
    }
}

pub fn init() {
    inject_scheduler(Box::leak(Box::new(FifoScheduler::default())));
    inject_pre_schedule_handler(pre_schedule_handler);
    inject_post_schedule_handler(post_schedule_handler);
}

/// A simple FIFO (First-In-First-Out) task scheduler.
struct FifoScheduler<T> {
    /// A thread-safe queue to hold tasks waiting to be executed.
    rq: Vec<SpinLock<FifoRunQueue<T>>>,
    queue: Arc<SpinLock<VecDeque<Arc<T>>>>,
}

impl<T> FifoScheduler<T> {
    /// Creates a new instance of `FifoScheduler`.
    fn new(nr_cpus: usize) -> Self {
        let queue = Arc::new(SpinLock::new(VecDeque::new()));

        let mut rq = Vec::new();
        for _ in 0..nr_cpus {
            rq.push(SpinLock::new(FifoRunQueue::new(queue.clone())));
        }
        Self { rq, queue }
    }
}

impl<T: CommonSchedInfo + Send + Sync> Scheduler<T> for FifoScheduler<T> {
    fn enqueue(&self, runnable: Arc<T>, _flags: EnqueueFlags) -> Option<CpuId> {
        let mut queue = self.queue.lock();

        for task in queue.iter() {
            if Arc::ptr_eq(task, &runnable) {
                return None;
            }
        }

        runnable.cpu().set_to_none();

        queue.push_back(runnable);

        None
    }

    fn local_rq_with(&self, f: &mut dyn FnMut(&dyn LocalRunQueue<T>)) {
        let preempt_guard = disable_preempt();
        let local_rq: &FifoRunQueue<T> = &self.rq[preempt_guard.current_cpu().as_usize()]
            .disable_irq()
            .lock();
        f(local_rq);
    }

    fn mut_local_rq_with(&self, f: &mut dyn FnMut(&mut dyn LocalRunQueue<T>)) {
        let preempt_guard = disable_preempt();
        let local_rq: &mut FifoRunQueue<T> = &mut self.rq[preempt_guard.current_cpu().as_usize()]
            .disable_irq()
            .lock();
        f(local_rq);
    }
}

struct FifoRunQueue<T> {
    current: Option<Arc<T>>,
    queue: Arc<SpinLock<VecDeque<Arc<T>>>>,
}

impl<T> FifoRunQueue<T> {
    pub fn new(queue: Arc<SpinLock<VecDeque<Arc<T>>>>) -> Self {
        Self {
            current: None,
            queue,
        }
    }
}

impl<T: CommonSchedInfo> LocalRunQueue<T> for FifoRunQueue<T> {
    fn current(&self) -> Option<&Arc<T>> {
        self.current.as_ref()
    }

    fn update_current(&mut self, _flags: UpdateFlags) -> bool {
        !self.queue.lock().is_empty()
    }

    fn try_pick_next(&mut self) -> Option<&Arc<T>> {
        let mut queue = self.queue.lock();

        let next_task = loop {
            let next_task = queue.pop_front()?;
            if next_task
                .cpu()
                .set_if_is_none(CpuId::current_racy())
                .is_ok()
            {
                break next_task;
            }
            queue.push_back(next_task);
        };

        if let Some(prev_task) = self.current.replace(next_task.clone()) {
            prev_task.cpu().set_to_none();
            queue.push_back(prev_task);
        }

        self.current.as_ref()
    }

    fn dequeue_current(&mut self) -> Option<Arc<T>> {
        self.current.take().inspect(|task| task.cpu().set_to_none())
    }
}

impl Default for FifoScheduler<Task> {
    fn default() -> Self {
        Self::new(num_cpus())
    }
}
