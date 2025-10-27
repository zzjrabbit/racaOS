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

            data.memory_info().vmar().write(address, &buf)?;

            *offset += len as u64;

            Ok(len as isize)
        })?
}

pub fn write(fd: FileDescriptor, address: Vaddr, len: usize) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    data.fs_info()
        .with_file_mut(fd, |offset, access_mode, _open_flags, file| {
            if !access_mode.is_writable() {
                return Err(Errno::EACCES.no_message());
            }

            let mut buffer = vec![0; len];
            data.memory_info().vmar().read(address, &mut buffer)?;

            let len = file.write_at(*offset, &buffer)?;
            *offset += len as u64;

            Ok(len as isize)
        })?
}

#[derive(Default, Clone, Copy, Pod)]
#[repr(C)]
struct IoVec {
    base: Vaddr,
    len: usize,
}

pub fn readv(fd: FileDescriptor, iov_address: Vaddr, count: usize) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    data.fs_info()
        .with_file_mut(fd, |offset, access_mode, _open_flags, file| {
            if !access_mode.is_readable() {
                return Err(Errno::EACCES.no_message());
            }

            let mut total_len = 0;

            for i in 0..count {
                let vec = data
                    .memory_info()
                    .vmar()
                    .read_val::<IoVec>(iov_address + (i * 2 * 8))?;

                let IoVec { base, len } = vec;

                let mut buffer = vec![0u8; len];

                let len = file.read_at(*offset, &mut buffer)?;

                data.memory_info().vmar().write(base, &buffer)?;

                *offset += len as u64;
                total_len += len;
            }

            Ok(total_len as isize)
        })?
}

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

                let buffer = (0..len)
                    .map(|id| data.memory_info().vmar().read_val::<u8>(base + id))
                    .collect::<Result<Vec<_>>>()?;

                let len = file.write_at(*offset, &buffer)?;

                *offset += len as u64;
                total_len += len;
            }

            Ok(total_len as isize)
        })?
}

pub fn pread64(fd: FileDescriptor, address: Vaddr, len: usize, offset: u64) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    data.fs_info()
        .with_file_mut(fd, |_offset, access_mode, _open_flags, file| {
            if !access_mode.is_readable() {
                return Err(Errno::EACCES.no_message());
            };

            let mut buf = vec![0; len];
            let len = file.read_at(offset, &mut buf)?;

            data.memory_info().vmar().write(address, &buf)?;

            Ok(len as isize)
        })?
}

pub fn pwrite64(fd: FileDescriptor, address: Vaddr, len: usize, offset: u64) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    data.fs_info()
        .with_file_mut(fd, |_offset, access_mode, _open_flags, file| {
            if !access_mode.is_writable() {
                return Err(Errno::EACCES.no_message());
            };

            let mut buf = vec![0; len];
            data.memory_info().vmar().read(address, &mut buf)?;

            let len = file.write_at(offset, &buf)?;

            Ok(len as isize)
        })?
}
