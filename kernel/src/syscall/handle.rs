use crate::{
    error::*,
    object::{HandleValue, Rights},
};

use super::current_process;

pub fn duplicate_handle(
    handle_value: HandleValue,
    rights: Rights,
    new_handle_value_ptr: *mut HandleValue,
) -> RcResult<usize> {
    let current_process = current_process();

    let new_handle_value =
        current_process.dup_handle_operating_rights(handle_value, |handle_rights| {
            if !handle_rights.contains(Rights::DUPLICATE) {
                return Err(RcError::AccessDenied);
            }
            if !rights.contains(Rights::SAME_RIGHTS) {
                if (handle_rights & rights).bits() != rights.bits() {
                    return Err(RcError::InvalidArguments);
                }
                Ok(rights)
            } else {
                Ok(handle_rights)
            }
        })?;

    unsafe {
        *new_handle_value_ptr = new_handle_value;
    }

    Ok(0)
}

pub fn close_handle(handle: HandleValue) -> RcResult<usize> {
    let current_process = current_process();
    current_process.remove_handle(handle)?;

    Ok(0)
}
