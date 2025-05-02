use core::slice::from_raw_parts;

use crate::{
    error::{RcError, RcResult},
    object::{Handle, HandleValue, INVALID_HANDLE, Rights},
    task::process::Process,
};

use super::current_process;

pub fn process_create(
    handle_ptr: usize,
    name_ptr: usize,
    name_len: usize,
    binary_ptr: usize,
    binary_len: usize,
    arg_handle_value: usize,
) -> RcResult<usize> {
    let name = unsafe { from_raw_parts(name_ptr as *const u8, name_len) };
    let name = core::str::from_utf8(name).map_err(|_| RcError::InvalidArguments)?;

    let binary = unsafe { from_raw_parts(binary_ptr as *const u8, binary_len) };

    let process = Process::create(name, binary)?;

    let current_process = current_process();

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
