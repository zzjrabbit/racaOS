use crate::task::ThreadData;

use super::*;
use ostd::{mm::Vaddr, task::Task};

pub fn get_tid() -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.data().downcast_ref::<ThreadData>().unwrap();
    Ok(data.tid() as isize)
}

pub fn set_tid_address(address: Vaddr) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.data().downcast_ref::<ThreadData>().unwrap();
    *data.tid_address.write() = Some(address);
    Ok(data.tid() as isize)
}

pub fn exit(_exit_code: i32) -> SyscallResult {
    let process = Process::current();
    ostd::early_println!("process exits");
    process.exit();
}
