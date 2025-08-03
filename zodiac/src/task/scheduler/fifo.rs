use alloc::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
};
use spin::{Lazy, RwLock};

use crate::{
    hal::cpu::Cpu,
    task::{LocalQueue, Scheduler, Thread},
};

pub struct FifoScheduler(Lazy<FifoSchedulerInner>);

struct FifoSchedulerInner {
    queue: Arc<RwLock<VecDeque<Arc<Thread>>>>,
    local_queues: RwLock<BTreeMap<Cpu, RwLock<FifoLocalQueue>>>,
}

impl FifoScheduler {
    pub const fn new() -> Self {
        FifoScheduler(Lazy::new(|| FifoSchedulerInner::new()))
    }
}

impl FifoSchedulerInner {
    fn new() -> Self {
        let queue = Arc::new(RwLock::new(VecDeque::new()));

        let mut local_queues = BTreeMap::new();
        for cpu in Cpu::all_cpus() {
            local_queues.insert(cpu, RwLock::new(FifoLocalQueue::new(queue.clone())));
        }

        FifoSchedulerInner {
            queue,
            local_queues: RwLock::new(local_queues),
        }
    }
}

impl Scheduler for FifoScheduler {
    fn retain(&self, f: &dyn Fn(&Arc<Thread>) -> bool) {
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
    current: Option<Arc<Thread>>,
    queue: Arc<RwLock<VecDeque<Arc<Thread>>>>,
}

impl FifoLocalQueue {
    pub const fn new(queue: Arc<RwLock<VecDeque<Arc<Thread>>>>) -> Self {
        FifoLocalQueue {
            current: None,
            queue,
        }
    }
}

impl LocalQueue for FifoLocalQueue {
    fn current(&self) -> Option<Arc<Thread>> {
        self.current.clone()
    }

    fn deque_next_thread(&mut self) -> Option<Arc<Thread>> {
        let thread = self.queue.write().pop_front();
        self.current = thread.clone();
        thread
    }

    fn enqueue(&mut self, thread: Arc<Thread>) {
        self.queue.write().push_back(thread);
    }
}
