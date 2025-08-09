use super::*;
use zodiac::{mem::VirtualAddress, task::Task};

pub fn set_tid_address(_address: VirtualAddress) -> SyscallResult {
    let thread = Task::current();
    Ok(thread.task_id() as isize)
}

pub fn exit(_exit_code: i32) -> SyscallResult {
    let process = Process::current();
    process.exit();
}
