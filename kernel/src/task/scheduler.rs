// TODO: MuQss

use core::{sync::atomic::AtomicBool, sync::atomic::Ordering};

use alloc::{
    collections::btree_map::BTreeMap,
    sync::{Arc, Weak},
    vec::Vec,
};
use spin::Mutex;
use x86_64::VirtAddr;

use crate::hal::{driver::apic::LAPIC, int::IntFrame, smp::CPUS};

use super::{process::Process, thread::Thread};

pub struct Scheduler {
    inner: Mutex<SchedulerInner>,
}

pub struct SchedulerInner {
    ready_threads: Vec<Weak<Thread>>,
    last_threads: BTreeMap<u32, Option<Weak<Thread>>>,
}

impl Scheduler {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(SchedulerInner {
                ready_threads: Vec::new(),
                last_threads: BTreeMap::new(),
            }),
        }
    }

    pub fn add_thread(&self, thread: &Arc<Thread>) {
        self.inner.lock().ready_threads.push(Arc::downgrade(thread));
    }

    pub fn add_cpu(&self, id: u32) {
        self.inner.lock().last_threads.insert(id, None);
    }

    pub fn current_thread(&self) -> Weak<Thread> {
        self.inner.lock().last_threads[&unsafe { LAPIC.lock().id() }]
            .clone()
            .unwrap()
    }

    pub fn schedule(&self, cpu_id: u32, context: &mut IntFrame) {
        let mut inner = self.inner.lock();

        if let Some(last_thread) = &inner.last_threads[&cpu_id] {
            if let Some(last_thread) = last_thread.upgrade() {
                last_thread.restore_context(context.clone());
                if !last_thread.sleeping() {
                    inner.ready_threads.push(Arc::downgrade(&last_thread));
                }
            }
        }

        let next_thread = inner
            .ready_threads
            .remove(0)
            .upgrade()
            .expect("Please remove killed threads.");

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

        Process::create("idle", include_bytes!(core::env!("CARGO_BIN_FILE_IDLE"))).unwrap();
    }

    log::info!("loading user_boot");

    Process::create(
        "user_boot",
        include_bytes!(core::env!("CARGO_BIN_FILE_USER_BOOT")),
    )
    .unwrap();

    SCHEDULER_INIT.store(true, Ordering::SeqCst);
}
