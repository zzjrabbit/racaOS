use crate::{AsThread, CloneArgs, UserThreadData, clone_child};

use super::*;
use alloc::{ffi::CString, sync::Arc, vec::Vec};
use ::filesystem::Path;
use memory::Vmar;
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

    let (_, child_process) = clone_child(CloneArgs::for_fork(), parent.cloned(), context)?;

    Ok(child_process.id() as isize)
}

pub fn execve(file_name: Vaddr, argv: Vaddr, envp: Vaddr) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();
    
    let vmar = data.memory_info().vmar();
    let file_name = Path::from(vmar.read_cstring(file_name, None)?.to_string_lossy().into_owned());
    let argv = read_cstring_array(vmar.clone(), argv, 128, 4096)?;
    let envp = read_cstring_array(vmar.clone(), envp, 128, 4096)?;

    let process = Process::current();
    //process.exec(file_name, argv, envp);

    Ok(0)
}

fn read_cstring_array(vmar: Arc<Vmar>, array: Vaddr, max_string_number: usize, max_string_len: usize) -> Result<Vec<CString>> {
    if array == 0 {
        return Ok(Vec::new());
    }
    
    let mut strings = Vec::new();
    
    for id in 0..max_string_number {
        let string_ptr = vmar.read_val(array + id * size_of::<u8>())?;
        if string_ptr == 0 {
            return Ok(strings);
        }
        
        let string = vmar.read_cstring(string_ptr, Some(max_string_len))?;
        strings.push(string);
    }
    
    Err(Errno::E2BIG.no_message())
}
