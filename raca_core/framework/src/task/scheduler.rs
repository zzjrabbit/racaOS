use core::sync::atomic::{AtomicBool, Ordering};

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::{Lazy, RwLock};
use x86_64::instructions::interrupts;
use x86_64::VirtAddr;

use super::context::Context;
use super::process::SharedProcess;
use super::thread::{SharedThread, ThreadState, WeakSharedThread};
use super::{Process, Thread};
use crate::arch::smp::CPUS;

pub static SCHEDULER_INIT: AtomicBool = AtomicBool::new(false);
pub static SCHEDULERS: RwLock<BTreeMap<u32, Scheduler>> = RwLock::new(BTreeMap::new());
pub static KERNEL_PROCESS: Lazy<SharedProcess> = Lazy::new(|| Process::new_kernel_process());
static PROCESSES: RwLock<Vec<SharedProcess>> = RwLock::new(Vec::new());
static THREADS: RwLock<Vec<WeakSharedThread>> = RwLock::new(Vec::new());

pub fn init() {
    add_process(KERNEL_PROCESS.clone());

    SCHEDULERS
        .write()
        .insert(CPUS.lock().bsp_id(), Scheduler::new());

    //x86_64::instructions::interrupts::enable();
    SCHEDULER_INIT.store(true, Ordering::Relaxed);
    log::info!("Scheduler initialized, interrupts enabled!");
}

#[inline]
pub fn add_process(process: SharedProcess) {
    interrupts::without_interrupts(|| {
        PROCESSES.write().push(process.clone());
    });
}

#[inline]
pub fn add_thread(thread: WeakSharedThread) {
    interrupts::without_interrupts(|| {
        log::info!("Add {}", thread.upgrade().unwrap().read().id.0);
        THREADS.write().push(thread);
    });
}

pub struct Scheduler {
    pub current_thread: SharedThread,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            current_thread: Thread::new_init_thread(),
        }
    }

    pub fn get_next(&mut self) -> SharedThread {
        let threads = THREADS.read();
        for thread in threads.iter() {
            let thread = thread.upgrade().unwrap();
            if thread.read().state == ThreadState::Ready {
                return thread.clone();
            }
        }
        unreachable!()
    }

    pub fn schedule(&mut self, context: VirtAddr) -> VirtAddr {
        let last_thread = {
            let mut thread = self.current_thread.write();
            thread.context = Context::from_address(context);
            self.current_thread.clone()
        };

        assert!(last_thread.read().state == ThreadState::Running);

        let _lock = crate::GLOBAL_MUTEX.lock();
        self.current_thread = self.get_next();
        self.current_thread.write().state = ThreadState::Running;

        last_thread.write().state = ThreadState::Ready;

        let next_thread = self.current_thread.read();

        let kernel_address = next_thread.kernel_stack.end_address();
        CPUS.lock().current_cpu().1.set_ring0_rsp(kernel_address);

        next_thread.context.address()
    }
}
