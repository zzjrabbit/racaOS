use super::*;
use zodiac::{mem::VirtualAddress, task::Thread};

pub fn set_tid_address(_address: VirtualAddress) -> SyscallResult {
    let thread = Thread::current();
    Ok(thread.thread_id() as isize)
}

pub fn exit(_exit_code: i32) -> SyscallResult {
    let process = Process::current();
    process.inner().exit();
}
