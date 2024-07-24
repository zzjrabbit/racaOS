use core::sync::atomic::{AtomicBool, Ordering};

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::{Lazy, Mutex, RwLock};
use x86_64::instructions::interrupts;
use x86_64::VirtAddr;

use super::context::Context;
use super::process::SharedProcess;
use super::thread::{SharedThread, ThreadState, WeakSharedThread};
use super::{Process, Thread};
use crate::arch::smp::CPUS;

pub static SCHEDULER_INIT: AtomicBool = AtomicBool::new(false);
pub static SCHEDULERS: Mutex<BTreeMap<u32, Scheduler>> = Mutex::new(BTreeMap::new());
pub static KERNEL_PROCESS: Lazy<SharedProcess> = Lazy::new(|| Process::new_kernel_process());
static PROCESSES: RwLock<Vec<SharedProcess>> = RwLock::new(Vec::new());
static THREADS: Mutex<Vec<WeakSharedThread>> = Mutex::new(Vec::new());

pub fn init() {
    add_process(KERNEL_PROCESS.clone());

    SCHEDULERS
        .lock()
        .insert(CPUS.lock().bsp_id(), Scheduler::new());

    //x86_64::instructions::interrupts::enable();
    SCHEDULER_INIT.store(true, Ordering::Relaxed);
    log::info!("Scheduler initialized!");
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
        THREADS.lock().push(thread);
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
        let mut threads = THREADS.lock();
        let mut idx = 0;
        while idx < threads.len() {
            let thread0 = threads.remove(0);
            let thread = thread0.upgrade().unwrap();
            threads.push(thread0);
            if thread.read().state == ThreadState::Ready {
                thread.write().state = ThreadState::Running;
                return thread.clone();
            }
            idx += 1;
        }
        unreachable!()
    }

    pub fn schedule(&mut self, context: VirtAddr) -> VirtAddr {
        //let _lock = crate::GLOBAL_MUTEX.lock();

        //assert_eq!(self.current_thread.read().state, ThreadState::Running);

        let last_thread = {
            let mut thread = self.current_thread.write();
            thread.context = Context::from_address(context);
            self.current_thread.clone()
        };

        self.current_thread = self.get_next();

        last_thread.write().state = ThreadState::Ready;

        let next_thread = self.current_thread.read();

        let kernel_address = next_thread.kernel_stack.end_address();
        CPUS.lock().current_cpu().1.set_ring0_rsp(kernel_address);

        next_thread.context.address()
    }
}
