mod fifo;

use alloc::{
    boxed::Box,
    sync::{Arc, Weak},
};
pub use fifo::*;
use spin::Once;

use crate::{
    hal::{
        context::TrapFrame,
        trap::{get_kernel_stack, get_user_stack, set_kernel_stack, set_user_stack},
    },
    task::{Task, TaskId},
};

pub trait Scheduler<T = Arc<Task>>: Sync + Send {
    fn with_local_queue(&self, f: &mut dyn FnMut(&dyn LocalQueue));
    fn with_local_queue_mut(&self, f: &mut dyn FnMut(&mut dyn LocalQueue));
    /// Retain the tasks that satisfy the predicate.
    fn retain(&self, f: &dyn Fn(&T) -> bool);
}

pub trait LocalQueue<T = Arc<Task>, W = Weak<Task>> {
    /// Returns the current task.
    /// Note that the current task is not in the ready queue.
    fn current(&self) -> Option<W>;
    /// Push a task into the local queue.
    fn enqueue(&mut self, task: T);
    /// Pick the next task to execute, and deque the task.
    fn deque_next_task(&mut self) -> Option<T>;
}

pub fn set_scheduler(scheduler: &'static dyn Scheduler) {
    SCHEDULER.set_scheduler(scheduler);
    crate::timer::register_callback(|ctx| schedule(ctx));
}

pub(crate) fn sheduler_check() {
    if !SCHEDULER.is_set() {
        set_scheduler(Box::leak(Box::new(fifo::FifoScheduler::new())));
    }
}

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
    fn is_set(&self) -> bool {
        self.scheduler.is_completed()
    }

    fn set_scheduler(&self, scheduler: &'static dyn Scheduler) {
        self.scheduler.call_once(|| scheduler);
    }
}

impl SchedulerWrapper {
    fn add(&self, task: Arc<Task>) {
        self.scheduler
            .get()
            .unwrap()
            .with_local_queue_mut(&mut |queue| queue.enqueue(task.clone()));
    }

    fn remove(&self, task_id: TaskId) {
        self.scheduler
            .get()
            .unwrap()
            .retain(&|t| t.task_id() != task_id);
    }

    fn schedule(&self, context: &mut TrapFrame) {
        self.scheduler
            .get()
            .unwrap()
            .with_local_queue_mut(&mut |queue| {
                if let Some(current) = queue.current()
                    && let Some(current) = current.upgrade()
                {
                    current.set_kernel_stack(get_kernel_stack());
                    current.set_user_stack(get_user_stack());
                    current.set_context(context.clone());
                    if !current.task_state().is_blocked() {
                        current.ready();
                        queue.enqueue(current.clone());
                    }
                }

                let next = queue.deque_next_task().expect("CPU Hungry.");
                next.run();
                *context = next.context();

                set_kernel_stack(next.kernel_stack());
                set_user_stack(next.user_stack());
            });
    }

    fn current(&self) -> Arc<Task> {
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

static PRE_SCHEDULE_HANDLER: Once<fn()> = Once::new();

static POST_SCHEDULE_HANDLER: Once<fn()> = Once::new();

pub fn set_pre_schedule_handler(handler: fn()) {
    PRE_SCHEDULE_HANDLER.call_once(|| handler);
}

pub fn set_post_schedule_handler(handler: fn()) {
    POST_SCHEDULE_HANDLER.call_once(|| handler);
}

mod wrappers {
    use super::*;

    pub fn add_task(task: Arc<Task>) {
        SCHEDULER.add(task);
    }

    pub fn remove_task(task_id: TaskId) {
        SCHEDULER.remove(task_id);
    }

    pub fn schedule(context: &mut TrapFrame) {
        if let Some(pre_schedule_handler) = PRE_SCHEDULE_HANDLER.get() {
            pre_schedule_handler();
        }
        SCHEDULER.schedule(context);
        if let Some(post_schedule_handler) = POST_SCHEDULE_HANDLER.get() {
            post_schedule_handler();
        }
    }

    pub fn current_task() -> Arc<Task> {
        SCHEDULER.current()
    }
}

pub(crate) use wrappers::*;
