use core::slice::from_raw_parts;

use crate::error::{RcError, RcResult};

pub fn debug(msg_ptr: usize, msg_len: usize) -> RcResult<usize> {
    let msg = unsafe { from_raw_parts(msg_ptr as *const u8, msg_len) };
    let msg = core::str::from_utf8(msg).map_err(|_| RcError::InvalidArguments)?;
    crate::print!("{}", msg);

    Ok(0)
}
