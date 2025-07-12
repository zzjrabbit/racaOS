// TODO: MuQss

use core::{sync::atomic::AtomicBool, sync::atomic::Ordering};

use alloc::{
    collections::btree_map::BTreeMap,
    sync::{Arc, Weak},
    vec::Vec,
};
use spin::RwLock;
use x86_64::VirtAddr;

use crate::{
    hal::{driver::apic::LAPIC, int::IntFrame, smp::CPUS},
    object::{Handle, KernelObject, KoID, Rights},
    task::job::ROOT_JOB,
};

use super::{
    process::Process,
    thread::{Thread, ThreadState},
};

pub struct Scheduler {
    inner: RwLock<SchedulerInner>,
}

pub struct SchedulerInner {
    ready_threads: Vec<Weak<Thread>>,
    last_threads: BTreeMap<u32, Option<Weak<Thread>>>,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            inner: RwLock::new(SchedulerInner {
                ready_threads: Vec::new(),
                last_threads: BTreeMap::new(),
            }),
        }
    }

    pub fn add_thread(&self, thread: &Arc<Thread>) {
        self.inner
            .write()
            .ready_threads
            .push(Arc::downgrade(thread));
    }

    pub fn remove_thread(&self, id: KoID) {
        self.inner.write().ready_threads.retain(|thread| {
            let thread = thread.upgrade();
            if let Some(thread) = thread {
                thread.id() != id
            } else {
                false
            }
        });
    }

    pub fn add_cpu(&self, id: u32) {
        self.inner.write().last_threads.insert(id, None);
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
                last_thread.restore_context(context.clone());
                if last_thread.state().running_on().is_some() {
                    last_thread.set_state(ThreadState::Ready);
                    inner.ready_threads.push(Arc::downgrade(&last_thread));
                } /* else {
                let process = last_thread.process().upgrade().unwrap();
                if let None = inner.last_threads.iter().find(|thread| )
                } */
            }
        }

        let next_thread = inner
            .ready_threads
            .remove(0)
            .upgrade()
            .expect("Please remove killed threads.");

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
