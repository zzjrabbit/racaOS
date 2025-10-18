use alloc::vec::Vec;
use ostd::{Pod, mm::Vaddr, task::Task};

use {
    crate::{AsThread, UserThreadData},
    ::filesystem::FileDescriptor,
};

use super::*;

pub fn read(fd: FileDescriptor, address: Vaddr, len: usize) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    data.fs_info()
        .with_file_mut(fd, |offset, access_mode, _open_flags, file| {
            if !access_mode.is_readable() {
                return Err(Errno::EACCES.no_message());
            };

            let mut buf = vec![0; len];
            let len = file.read_at(*offset, &mut buf)?;

            for (id, byte) in buf.iter().enumerate() {
                data.memory_info().vmar().write_val(address + id, byte)?;
            }

            *offset += len as u64;

            Ok(len as isize)
        })
        .ok_or(Errno::EBADFD.no_message())?
}

pub fn write(fd: FileDescriptor, address: Vaddr, len: usize) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    data.fs_info()
        .with_file_mut(fd, |offset, access_mode, _open_flags, file| {
            if !access_mode.is_writable() {
                return Err(Errno::EACCES.no_message());
            }

            let buffer = (0..len)
                .map(|id| data.memory_info().vmar().read_val::<u8>(address + id))
                .collect::<Result<Vec<_>>>()?;

            let len = file.write_at(*offset, &buffer)?;
            *offset += len as u64;

            Ok(len as isize)
        })
        .ok_or(Errno::EBADFD.no_message())?
}

#[derive(Default, Clone, Copy)]
#[repr(C)]
struct IoVec {
    base: Vaddr,
    len: usize,
}

#[allow(unsafe_code)]
unsafe impl Pod for IoVec {}

pub fn writev(fd: FileDescriptor, iov_address: Vaddr, count: usize) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    data.fs_info()
        .with_file_mut(fd, |offset, access_mode, _open_flags, file| {
            if !access_mode.is_writable() {
                return Err(Errno::EACCES.no_message());
            }

            let mut total_len = 0;

            for i in 0..count {
                let vec = data
                    .memory_info()
                    .vmar()
                    .read_val::<IoVec>(iov_address + (i * 2 * 8))?;

                let IoVec { base, len } = vec;

                if len > 0xffffff {
                    return Ok(0);
                }

                let buffer = (0..len)
                    .map(|id| data.memory_info().vmar().read_val::<u8>(base + id))
                    .collect::<Result<Vec<_>>>()?;

                let len = file.write_at(*offset, &buffer)?;

                *offset += len as u64;
                total_len += len;
            }

            Ok(total_len as isize)
        })
        .ok_or(Errno::ENOENT.no_message())?
}
