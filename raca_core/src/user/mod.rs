use alloc::sync::Arc;
use framework::task::{process::ProcessId, scheduler::SCHEDULER, thread::ThreadState, Process, Thread};
use spin::RwLock;

pub mod login;
pub mod syscall;

#[inline]
pub fn get_current_thread() -> Arc<RwLock<Thread>> {
    SCHEDULER.lock().current_thread().upgrade().unwrap()
}

#[inline]
pub fn get_current_process() -> Arc<RwLock<Process>> {
    get_current_thread().read().process.upgrade().unwrap()
}

#[inline]
pub fn get_current_process_id() -> ProcessId {
    get_current_process().read().id
}

#[inline]
pub fn sleep() {
    get_current_thread().write().state = ThreadState::Blocked;
    framework::task::schedule();
    while get_current_thread().read().state == ThreadState::Blocked {}
}
