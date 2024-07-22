#[repr(C)]
pub enum OpenMode {
    Read = 0,
    Write = 1,
}

pub type FileDescriptor = usize;

pub fn open(path: &str, open_mode: OpenMode) -> Result<FileDescriptor, ()> {
    const OPEN_SYSCALL_ID: u64 = 2;
    let fd = crate::syscall(
        OPEN_SYSCALL_ID,
        path.as_ptr() as usize,
        path.len(),
        open_mode as usize,
        0,
        0,
    );
    if fd == 0 {
        Err(())
    } else {
        Ok(fd)
    }
}

pub fn read(file_descriptor: FileDescriptor, buffer: &mut [u8]) -> usize {
    const READ_SYSCALL_ID: u64 = 4;
    crate::syscall(
        READ_SYSCALL_ID,
        file_descriptor,
        buffer.as_ptr() as usize,
        buffer.len(),
        0,
        0,
    )
}

pub fn write(file_descriptor: FileDescriptor, buffer: &[u8]) -> usize {
    const WRITE_SYSCALL_ID: u64 = 3;
    crate::syscall(
        WRITE_SYSCALL_ID,
        file_descriptor,
        buffer.as_ptr() as usize,
        buffer.len(),
        0,
        0,
    )
}

pub fn close(file_descriptor: FileDescriptor) -> usize {
    const CLOSE_SYSCALL_ID: u64 = 9;
    crate::syscall(CLOSE_SYSCALL_ID, file_descriptor, 0, 0, 0, 0)
}

pub fn lseek(file_descriptor: FileDescriptor, offset: usize) -> usize {
    const LSEEK_SYSCALL_ID: u64 = 10;
    crate::syscall(LSEEK_SYSCALL_ID, file_descriptor, offset, 0, 0, 0)
}
