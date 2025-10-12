use crate::{clone_child, AsThread, CloneArgs, UserThreadData};

use super::*;
use ostd::{mm::Vaddr, task::Task};

pub fn get_tid() -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();
    Ok(data.tid() as isize)
}

pub fn set_tid_address(address: Vaddr) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();
    *data.tid_address.write() = Some(address);
    Ok(data.tid() as isize)
}

pub fn exit(exit_code: i32) -> SyscallResult {
    let process = Process::current();
    process.exit(exit_code);

    Ok(0)
}

pub fn fork(context: &UserContext) -> SyscallResult {
    let parent = Task::current().unwrap();

    let (_, child_process) = clone_child(CloneArgs::default(), parent.cloned(), context)?;

    Ok(child_process.id() as isize)
}
