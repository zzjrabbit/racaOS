use crate::{RcResult, right::Rights, syscall};

pub fn duplicate_handle(handle: u32, rights: Rights) -> RcResult<u32> {
    let mut new_handle = 0u32;

    syscall!(14, handle, rights.bits(), &mut new_handle)?;

    Ok(new_handle)
}

pub fn close_handle(handle: u32) -> RcResult<()> {
    syscall!(30, handle)?;

    Ok(())
}
