use core::str;

use crate::error::{RcError, RcResult};

pub fn debug(ptr: usize, len: usize) -> RcResult<()> {
    let data = unsafe { core::slice::from_raw_parts(ptr as *const u8, len) };
    crate::print!(
        "{}",
        str::from_utf8(data).map_err(|_| RcError::INVALID_ARGS)?
    );

    Ok(())
}
