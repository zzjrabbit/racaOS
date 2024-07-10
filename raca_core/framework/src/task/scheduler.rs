use core::sync::atomic::{AtomicBool, Ordering};

use alloc::collections::VecDeque;
use spin::{Lazy, RwLock};
use x86_64::VirtAddr;

use super::context::Context;
use super::process::SharedProcess;
use super::thread::{SharedThread, ThreadState};
use super::{Process, Thread};
use crate::arch::smp::CPUS;

pub static SCHEDULER_INIT: AtomicBool = AtomicBool::new(false);
pub static SCHEDULER: Lazy<RwLock<Scheduler>> = Lazy::new(|| RwLock::new(Scheduler::new()));
pub static KERNEL_PROCESS: Lazy<SharedProcess> = Lazy::new(|| Process::new_kernel_process());

pub fn init() {
    let idle_thread = || loop {
        x86_64::instructions::hlt();
    };
    Thread::new_kernel_thread(idle_thread);

    SCHEDULER.write().add(KERNEL_PROCESS.clone());
    //x86_64::instructions::interrupts::enable();
    SCHEDULER_INIT.store(true, Ordering::Relaxed);
    log::info!("Scheduler initialized, interrupts enabled!");
}

pub struct Scheduler {
    pub current_thread: SharedThread,
    processes: VecDeque<SharedProcess>,
    last_scheduled: (usize, usize),
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            current_thread: Thread::new_init_thread(),
            processes: VecDeque::new(),
            last_scheduled: (0, 0),
        }
    }

    #[inline]
    pub fn add(&mut self, process: SharedProcess) {
        self.processes.push_back(process);
    }

    pub fn get_next(&mut self) -> SharedThread {
        let process_count = self.processes.len();
        if process_count == 0 {
            panic!("No processes to schedule!");
        }

        let (mut process_index, mut thread_index) = self.last_scheduled;

        for _ in 0..process_count {
            process_index = (process_index + 1) % process_count;
            let process = self.processes[process_index].read();
            let thread_count = process.threads.len();
            if thread_count == 0 {
                continue;
            }

            for _ in 0..thread_count {
                thread_index = (thread_index + 1) % thread_count;
                let thread = &process.threads[thread_index];
                let mut thread_mut = thread.write();
                if thread_mut.state == ThreadState::Ready {
                    thread_mut.state = ThreadState::Running;
                    // log::debug!("Thread {:?} is ready", thread_mut.id);
                    drop(thread_mut);
                    self.last_scheduled = (process_index, thread_index); // 更新 last_scheduled
                    return thread.clone();
                }
            }
        }

        panic!("No threads to schedule!");
    }

    pub fn schedule(&mut self, context: VirtAddr) -> VirtAddr {
        let next_thread = self.get_next();

        {
            let mut thread = self.current_thread.write();
            // log::debug!("Switching from thread {:?}", thread.id);
            thread.context = Context::from_address(context);
            thread.state = ThreadState::Ready;
        }

        self.current_thread = next_thread.clone();
        let next_thread = next_thread.read();
        // log::warn!("next thread state: {:?}", next_thread.state);

        let kernel_address = next_thread.kernel_stack.end_address();
        CPUS.lock().current_cpu().set_ring0_rsp(kernel_address);

        next_thread.context.address()
    }
}
