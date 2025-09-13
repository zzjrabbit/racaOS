use alloc::{
    collections::{BTreeMap, VecDeque},
    sync::{Arc, Weak},
};
use spin::{Lazy, RwLock};
use zodiac::{
    hal::{cpu::Cpu, write_fs, write_gs},
    task::{LocalQueue, Scheduler, Task, set_post_schedule_handler, set_scheduler},
};

use crate::task::ThreadData;

static SCHEDULER: FifoScheduler = FifoScheduler::new();

fn post_schedule_handler() {
    let task = Task::current();
    if let Some(data) = task.data().downcast_ref::<ThreadData>() {
        data.vm_space.switch();
        if let Some(fs) = *data.fs.read() {
            write_fs(fs);
        }
        if let Some(gs) = *data.gs.read() {
            write_gs(gs);
        }
    }
}

pub fn init() {
    set_post_schedule_handler(post_schedule_handler);
    set_scheduler(&SCHEDULER);
}

pub struct FifoScheduler(Lazy<FifoSchedulerInner>);

struct FifoSchedulerInner {
    queue: Arc<RwLock<VecDeque<Arc<Task>>>>,
    local_queues: RwLock<BTreeMap<Cpu, RwLock<FifoLocalQueue>>>,
}

impl FifoScheduler {
    pub const fn new() -> Self {
        FifoScheduler(Lazy::new(FifoSchedulerInner::new))
    }
}

impl Default for FifoScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl FifoSchedulerInner {
    fn new() -> Self {
        let queue = Arc::new(RwLock::new(VecDeque::new()));

        let mut local_queues = BTreeMap::new();
        for cpu in Cpu::all() {
            local_queues.insert(cpu, RwLock::new(FifoLocalQueue::new(queue.clone())));
        }

        FifoSchedulerInner {
            queue,
            local_queues: RwLock::new(local_queues),
        }
    }
}

impl Scheduler for FifoScheduler {
    fn retain(&self, f: &dyn Fn(&Arc<Task>) -> bool) {
        self.0.queue.write().retain(f);
    }

    fn with_local_queue(&self, f: &mut dyn FnMut(&dyn LocalQueue)) {
        let current_cpu = Cpu::current();

        let queues = self.0.local_queues.read();
        let queue = queues.get(&current_cpu).unwrap();
        let queue = queue.read();
        f(&*queue);
    }

    fn with_local_queue_mut(&self, f: &mut dyn FnMut(&mut dyn LocalQueue)) {
        let current_cpu = Cpu::current();

        let queues = self.0.local_queues.read();
        let queue = queues.get(&current_cpu).unwrap();
        let mut queue = queue.write();
        f(&mut *queue);
    }
}

pub struct FifoLocalQueue {
    current: Option<Weak<Task>>,
    queue: Arc<RwLock<VecDeque<Arc<Task>>>>,
}

impl FifoLocalQueue {
    pub const fn new(queue: Arc<RwLock<VecDeque<Arc<Task>>>>) -> Self {
        FifoLocalQueue {
            current: None,
            queue,
        }
    }
}

impl LocalQueue for FifoLocalQueue {
    fn current(&self) -> Option<Weak<Task>> {
        self.current.clone()
    }

    fn deque_next_task(&mut self) -> Option<Arc<Task>> {
        let task = self.queue.write().pop_front()?;
        self.current = Some(Arc::downgrade(&task));

        Some(task)
    }

    fn enqueue(&mut self, task: Arc<Task>) {
        self.queue.write().push_back(task);
    }
}
