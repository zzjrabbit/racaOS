use crate::RcResult;

pub fn debug(msg: &str) -> RcResult<()> {
    crate::syscall!(0, msg.as_ptr() as usize, msg.len())?;
    Ok(())
}
