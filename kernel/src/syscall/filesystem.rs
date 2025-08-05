use zodiac::mem::VirtualAddress;

use super::*;

pub fn write(_fd: usize, address: VirtualAddress, len: usize) -> SyscallResult {
    let current_process = Process::current();

    let mut buffer = vec![0; len];
    current_process
        .inner()
        .vm_space()
        .reader(address, len)
        .read(&mut buffer)?;

    crate::terminal::terminal_write(
        core::str::from_utf8(&buffer)
            .map_err(|_| SyscallError::InvalidArguments)?
            .to_string(),
    );

    Ok(len as isize)
}