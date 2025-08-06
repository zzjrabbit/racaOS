use zodiac::mem::VirtualAddress;

use crate::task::OpenMode;

use super::*;

pub fn read(fd: u32, address: VirtualAddress, len: usize) -> SyscallResult {
    let current_process = Process::current();

    let (offset, open_mode, file) = current_process.get_file(fd).ok_or(ZodiacError::NotFound)?;
    
    if matches!(open_mode, OpenMode::Write) {
        return Err(SyscallError::PermissionDenied);
    }
    
    let mut buf = vec![0; len];
    let len = file.read_at(offset, &mut buf);
    
    current_process.inner().vm_space().writer(address, len).write(&buf)?;
    
    Ok(len as isize)
}

pub fn write(fd: u32, address: VirtualAddress, len: usize) -> SyscallResult {
    let current_process = Process::current();

    let (offset, open_mode, file) = current_process.get_file(fd).ok_or(ZodiacError::NotFound)?;
    
    if matches!(open_mode, OpenMode::Read) {
        return Err(SyscallError::PermissionDenied);
    }
    
    let mut buffer = vec![0; len];
    
    current_process.inner().vm_space().reader(address, len).read(&mut buffer)?;
    
    Ok(file.write_at(offset, &buffer) as isize)
}
