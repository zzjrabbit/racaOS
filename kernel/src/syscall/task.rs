use core::slice::from_raw_parts;

use crate::{
    error::{RcError, RcResult},
    mm::VirtualMemory,
    object::{Handle, HandleValue, INVALID_HANDLE, Rights},
    task::{
        job::Job,
        job_policy::{BasicPolicy, SetPolicyOptions},
        process::{Process, ProcessInfo},
        scheduler::SCHEDULER,
        thread::Thread,
    },
};

use super::current_process;

#[repr(C)]
pub struct ProcessCreationArguments {
    name_ptr: *const u8,
    name_len: usize,
    binary_ptr: *const u8,
    binary_len: usize,
}

pub fn process_create(
    job_handle: usize,
    handle_ptr: usize,
    args: *const ProcessCreationArguments,
    arg_handle_value: usize,
) -> RcResult<usize> {
    let current_process = current_process();

    let args = unsafe { &*args };

    let name = unsafe { from_raw_parts(args.name_ptr, args.name_len) };
    let name = core::str::from_utf8(name).map_err(|_| RcError::InvalidArguments)?;

    let binary = unsafe { from_raw_parts(args.binary_ptr, args.binary_len) };

    let job =
        current_process.get_object_with_rights(job_handle as HandleValue, Rights::MANAGE_JOB)?;

    let process = Process::create(&job, name, binary)?;

    if arg_handle_value as u32 != INVALID_HANDLE {
        let arg_handle = current_process.remove_handle(arg_handle_value as HandleValue)?;
        process.add_handle(arg_handle);
    }

    let handle = current_process.add_handle(Handle::new(process, Rights::DEFAULT_PROCESS));

    unsafe {
        *(handle_ptr as *mut HandleValue) = handle as HandleValue;
    }

    Ok(0)
}

pub fn process_kill(handle: HandleValue) -> RcResult<usize> {
    let current_process = current_process();

    let process =
        current_process.get_object_with_rights::<Process>(handle, Rights::MANAGE_PROCESS)?;
    process.kill();

    Ok(0)
}

pub fn job_create(father_job_handle: HandleValue, handle_ptr: *mut HandleValue) -> RcResult<usize> {
    let current_process = current_process();

    let father_job =
        current_process.get_object_with_rights::<Job>(father_job_handle, Rights::MANAGE_JOB)?;

    let job = father_job.create_child()?;
    let handle_value = current_process.add_handle(Handle::new(job, Rights::DEFAULT_JOB));

    unsafe {
        handle_ptr.write(handle_value);
    }

    Ok(0)
}

pub fn job_set_policy(
    handle: HandleValue,
    policy_ptr: *const BasicPolicy,
    policy_len: usize,
) -> RcResult<usize> {
    let current_process = current_process();

    let job = current_process.get_object_with_rights::<Job>(handle, Rights::MANAGE_JOB)?;
    let policies = unsafe { from_raw_parts(policy_ptr, policy_len) };

    job.set_policy_basic(SetPolicyOptions::Relative, policies)?;

    Ok(0)
}

pub fn job_kill(handle: HandleValue) -> RcResult<usize> {
    let current_process = current_process();

    let job = current_process.get_object_with_rights::<Job>(handle, Rights::MANAGE_JOB)?;

    job.kill();

    Ok(0)
}

pub fn exit(code: i64) -> RcResult<usize> {
    let current_process = current_process();

    current_process.exit(code);

    Ok(0)
}

pub fn get_process_info(process: HandleValue, addr: *mut ProcessInfo) -> RcResult<usize> {
    let current_process = current_process();

    let process = current_process.get_object_with_rights::<Process>(process, Rights::GET_INFO)?;
    unsafe {
        addr.write(process.get_info());
    }

    Ok(0)
}

pub fn spawn_thread(
    name_ptr: *const u8,
    name_len: usize,
    entry: usize,
    stack_vm_handle: HandleValue,
    handle: *mut HandleValue,
) -> RcResult<usize> {
    let current_process = current_process();

    let name = unsafe { from_raw_parts(name_ptr, name_len) };
    let name = core::str::from_utf8(name).map_err(|_| RcError::InvalidArguments)?;

    let stack = current_process
        .get_object_with_rights::<VirtualMemory>(stack_vm_handle, Rights::empty())?;

    let thread = Thread::create(&current_process, name, entry, stack)?;
    let thread_handle = current_process.add_handle(Handle::new(thread, Rights::DEFAULT_THREAD));

    unsafe {
        handle.write(thread_handle);
    }

    Ok(0)
}

pub fn exit_thread() -> RcResult<usize> {
    let current_thread = SCHEDULER.current_thread().upgrade().unwrap();

    current_thread.exit();

    unsafe {
        core::arch::asm!("int 0x21");
    }

    Ok(0)
}

pub fn kill_thread(handle: HandleValue) -> RcResult<usize> {
    let current_process = current_process();

    let thread = current_process.get_object_with_rights::<Thread>(handle, Rights::MANAGE_THREAD)?;

    thread.kill();

    Ok(0)
}
