use alloc::{
    collections::{BTreeMap, VecDeque},
    sync::{Arc, Weak},
};
use spin::{Lazy, RwLock};
use zodiac::task::{set_scheduler, Task};

use crate::{
    hal::cpu::Cpu,
    task::{LocalQueue, Scheduler, Task},
};

static SCHEDULER: FifoScheduler = FifoScheduler::new();

pub fn init() {
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
        let Some(data)task.data();
        Some(task)
    }

    fn enqueue(&mut self, task: Arc<Task>) {
        self.queue.write().push_back(task);
    }
}

