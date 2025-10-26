use core::str::FromStr;

use crate::{AsThread, CloneArgs, MemoryInfo, UserStack, UserThreadData, clone_child};

use super::*;
use ::filesystem::Path;
use alloc::{ffi::CString, sync::Arc, vec::Vec};
use credentials::Uid;
use memory::Vmar;
use ostd::{arch::cpu::context::GeneralRegs, mm::Vaddr, task::Task, user::UserContextApi};

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

pub fn execve(
    file_name: Vaddr,
    argv: Vaddr,
    envp: Vaddr,
    context: &mut UserContext,
) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    let vmar = data.memory_info().vmar();
    let file_name = Path::from(
        vmar.read_cstring(file_name, None)?
            .to_string_lossy()
            .into_owned(),
    );
    let argv = read_cstring_array(vmar.clone(), argv, 128, 4096)?;
    let envp = read_cstring_array(vmar.clone(), envp, 128, 4096)?;

    let new_memory_info = Arc::new(MemoryInfo::new(Vmar::new()));

    let mut user_stack = UserStack::new(&new_memory_info);

    let (entry, aux_vec) = {
        let file = data
            .open_file(&file_name)
            .ok_or(Errno::ENOENT.no_message())?;
        let mut buffer = alloc::vec![0u8; file.len() as usize];
        file.read_at(0, &mut buffer)?;

        let exec_file_name =
            user_stack.push_a_lot(CString::from_str(&file_name).unwrap().as_bytes_with_nul());

        new_memory_info.load(exec_file_name, Some(&data), &buffer)?
    };

    let stack_pointer = {
        let mut envp = envp
            .iter()
            .map(|cstring| user_stack.push_a_lot(cstring.as_bytes_with_nul()))
            .collect::<Vec<_>>();
        envp.push(0);

        let argv = argv
            .iter()
            .map(|cstring| user_stack.push_a_lot(cstring.as_bytes_with_nul()))
            .collect::<Vec<_>>();

        user_stack.push_zero_until_aligned(16);

        user_stack.push_a_lot(&aux_vec.as_slice());

        user_stack.push_a_lot(&envp);
        user_stack.push(0usize);
        user_stack.push_a_lot(&argv);
        user_stack.push(argv.len());

        user_stack.stack_pointer()
    };

    *context.general_regs_mut() = GeneralRegs::default();
    context.set_tls_pointer(0);
    context.set_instruction_pointer(entry);
    context.set_stack_pointer(stack_pointer);

    data.replace_memory_info(new_memory_info.clone());
    new_memory_info.vmar().activate();

    data.fs_info().close_on_execve();

    Ok(0)
}

fn read_cstring_array(
    vmar: Arc<Vmar>,
    array: Vaddr,
    max_string_number: usize,
    max_string_len: usize,
) -> Result<Vec<CString>> {
    if array == 0 {
        return Ok(Vec::new());
    }

    let mut strings = Vec::new();
    let mut read = 0;

    for _ in 0..max_string_number {
        let string_ptr = vmar.read_val(array + read)?;
        if string_ptr == 0 {
            return Ok(strings);
        }

        let string = vmar.read_cstring(string_ptr, Some(max_string_len))?;
        strings.push(string);
        read += size_of::<usize>();
    }

    Err(Errno::E2BIG.no_message())
}

pub fn sched_yield() -> SyscallResult {
    Task::yield_now();
    Ok(0)
}

pub fn wait4(_pid: u64, status: Vaddr, _options: u32, _rusage: Vaddr) -> SyscallResult {
    let done = |child: Arc<Process>, exit_code: i32| -> Result<()> {
        let thread = Task::current().unwrap();
        let data = thread.direct_downcast::<UserThreadData>().unwrap();

        data.memory_info().vmar().write_val(status, &exit_code)?;

        child.clear_zombie();

        Ok(())
    };

    let process = Process::current();
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    if let Some((child, exit_code)) = process
        .children()
        .iter()
        .map(|child| (child.clone(), child.status().exit_code()))
        .find(|(_, exit_code)| exit_code.is_some())
    {
        done(child, exit_code.unwrap())?;
    };

    let (child, exit_code) = data.wait_with_waker(
        || {
            process
                .children()
                .iter()
                .map(|child| (child.clone(), child.status().exit_code()))
                .find(|(_, exit_code)| exit_code.is_some())
        },
        |waker| {
            for child in process.children() {
                child.add_zombie_waker(waker.clone());
            }
        },
    )?;
    let exit_code = exit_code.unwrap();

    done(child.clone(), exit_code)?;

    Ok(child.id() as isize)
}

pub fn getpid() -> SyscallResult {
    let process = Process::current();
    Ok(process.id() as isize)
}

pub fn getuid() -> SyscallResult {
    Ok(u32::from(Uid::new_root()) as isize)
}

pub fn getppid() -> SyscallResult {
    let process = Process::current();
    Ok(process.parent().unwrap().id() as isize)
}
