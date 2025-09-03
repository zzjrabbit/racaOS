use crate::task::ThreadData;

use super::*;
use zodiac::{mem::VirtualAddress, task::Task};

pub fn get_tid() -> SyscallResult {
    let thread = Task::current();
    Ok(thread.task_id() as isize)
}

pub fn set_tid_address(address: VirtualAddress) -> SyscallResult {
    let thread = Task::current();
    let data = thread.data().downcast_ref::<ThreadData>().unwrap();
    *data.tid_address.write() = Some(address);
    Ok(thread.task_id() as isize)
}

pub fn exit(_exit_code: i32) -> SyscallResult {
    let process = Process::current();
    process.exit();
}
