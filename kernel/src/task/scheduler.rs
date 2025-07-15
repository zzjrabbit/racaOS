// TODO: MuQss

use core::sync::atomic::{AtomicBool, Ordering};

use alloc::{
    collections::{binary_heap::BinaryHeap, btree_map::BTreeMap},
    sync::{Arc, Weak},
};
use spin::{Mutex, MutexGuard, RwLock};
use x86_64::VirtAddr;

use crate::{
    hal::{driver::apic::LAPIC, int::IntFrame, smp::CPUS}, object::{Handle, KernelObject, KoID, Rights}, print, println, task::job::ROOT_JOB
};

use super::{
    process::Process,
    thread::{Thread, ThreadState},
};

pub struct Scheduler {
    inner: RwLock<SchedulerInner>,
}

#[derive(Clone)]
struct ThreadWrapper(Weak<Thread>);

impl PartialEq for ThreadWrapper {
    fn eq(&self, other: &Self) -> bool {
        if let Some(this) = self.0.upgrade() {
            if let Some(other) = other.0.upgrade() {
                this.eq(&other)
            } else {
                false
            }
        } else {
            if let Some(_) = other.0.upgrade() {
                false
            } else {
                true
            }
        }
    }
}

impl Eq for ThreadWrapper {}

impl PartialOrd for ThreadWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(if let Some(this) = self.0.upgrade() {
            if let Some(other) = other.0.upgrade() {
                this.cmp(&other)
            } else {
                core::cmp::Ordering::Greater
            }
        } else {
            core::cmp::Ordering::Less
        })
    }
}

impl Ord for ThreadWrapper {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub struct SchedulerInner {
    ready_threads: BTreeMap<u32, Mutex<BinaryHeap<ThreadWrapper>>>,
    last_threads: BTreeMap<u32, Option<Weak<Thread>>>,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            inner: RwLock::new(SchedulerInner {
                ready_threads: BTreeMap::new(),
                last_threads: BTreeMap::new(),
            }),
        }
    }

    pub fn add_thread(&self, thread: &Arc<Thread>) {
        let cpu_id = unsafe { LAPIC.lock().id() };

        self.inner
            .write()
            .ready_threads
            .get(&cpu_id)
            .unwrap()
            .lock()
            .push(ThreadWrapper(Arc::downgrade(thread)));
    }

    pub fn remove_thread(&self, id: KoID) {
        self.inner
            .write()
            .ready_threads
            .iter()
            .for_each(|(_, list)| {
                list.lock().retain(|thread| {
                    let thread = thread.0.upgrade();
                    if let Some(thread) = thread {
                        thread.id() != id
                    } else {
                        false
                    }
                })
            });
    }

    pub fn add_cpu(&self, id: u32) {
        self.inner.write().last_threads.insert(id, None);
        self.inner
            .write()
            .ready_threads
            .insert(id, Mutex::new(BinaryHeap::new()));
    }

    pub fn current_thread(&self) -> Weak<Thread> {
        self.inner.read().last_threads[&unsafe { LAPIC.lock().id() }]
            .clone()
            .unwrap()
    }

    pub fn schedule(&self, cpu_id: u32, context: &mut IntFrame) {
        let mut inner = self.inner.write();

        if let Some(last_thread) = &inner.last_threads[&cpu_id] {
            if let Some(last_thread) = last_thread.upgrade() {
                last_thread.update_virtual_deadline();
                last_thread.restore_context(context.clone());
                if last_thread.state().running_on().is_some() {
                    last_thread.set_state(ThreadState::Ready);
                    inner
                        .ready_threads
                        .get_mut(&cpu_id)
                        .unwrap()
                        .lock()
                        .push(ThreadWrapper(Arc::downgrade(&last_thread)));
                } /* else {
                let process = last_thread.process().upgrade().unwrap();
                if let None = inner.last_threads.iter().find(|thread| )
                } */
            }
        }
        
        let get_next_thread = || {
            let mut next_thread: Option<Arc<Thread>> = None;
            let mut queue_guard: Option<MutexGuard<'_, _>> = None;

            for (_, queue) in inner.ready_threads.iter() {
                if let Some(queue) = queue.try_lock() {
                    if queue.is_empty() {
                        continue;
                    }

                    let Some(current) = queue.peek().unwrap().0.upgrade() else {
                        continue;
                    };

                    match next_thread {
                        Some(ref thread) => {
                            if thread.virtual_deadline() > current.virtual_deadline() {
                                queue_guard = Some(queue);
                                next_thread = Some(current.clone());
                            }
                        }
                        None => {
                            queue_guard = Some(queue);
                            next_thread = Some(current.clone());
                        }
                    }
                }
            }

            (next_thread, queue_guard)
        };

        let mut next_thread = None;
        let mut queue_guard = None;
        let mut count = 0;
        while let None = next_thread {
            (next_thread, queue_guard) = get_next_thread();
            count += 1;
        }

        print!("[{}]", count);

        let next_thread = next_thread.unwrap();
        let mut queue = queue_guard.unwrap();

        queue.pop();
        drop(queue);

        next_thread.set_state(ThreadState::RunningOn(cpu_id));

        next_thread.load_context(context);

        inner
            .last_threads
            .insert(cpu_id, Some(Arc::downgrade(&next_thread)));

        CPUS.write()
            .get_mut(cpu_id)
            .set_ring0_rsp(VirtAddr::new(next_thread.kernel_stack() as u64));

        let process = next_thread
            .process()
            .upgrade()
            .expect("Please kill the threads after killing the process");
        process.vmar().load_page_table();
    }
}

pub static SCHEDULER_INIT: AtomicBool = AtomicBool::new(false);
pub static SCHEDULER: Scheduler = Scheduler::new();

pub fn init() {
    for id in CPUS.read().iter_id() {
        SCHEDULER.add_cpu(*id);

        Process::create(
            &ROOT_JOB,
            "idle",
            include_bytes!(core::env!("CARGO_BIN_FILE_IDLE")),
        )
        .unwrap();
    }

    log::info!("loading user_boot");

    let user_boot = Process::create(
        &ROOT_JOB,
        "user_boot",
        include_bytes!(core::env!("CARGO_BIN_FILE_USER_BOOT")),
    )
    .unwrap();

    let _ = user_boot.add_handle(Handle::new(ROOT_JOB.clone(), Rights::DEFAULT_JOB));

    SCHEDULER_INIT.store(true, Ordering::SeqCst);
}
