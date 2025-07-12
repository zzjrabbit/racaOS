use crate::{
    error::RcResult,
    object::{HandleValue, Rights, Signal},
};

use super::current_process;

pub fn set_signal(handle: HandleValue, signal: Signal) -> RcResult<usize> {
    let current_process = current_process();

    let object = current_process.get_object_with_rights_no_downgrade(handle, Rights::GET_INFO)?;
    object.set_signal(signal);

    Ok(0)
}

pub fn clear_signal(handle: HandleValue, signal: Signal) -> RcResult<usize> {
    let current_process = current_process();

    let object = current_process.get_object_with_rights_no_downgrade(handle, Rights::GET_INFO)?;
    object.clear_signal(signal);

    Ok(0)
}
