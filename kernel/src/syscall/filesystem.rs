use alloc::vec::Vec;
use ostd::{mm::Vaddr, task::Task, Pod};

use crate::{
    filesystem::{open_file, AccessMode, FileDescriptor, FileType, InodeMode, OpenFlags, Path},
    task::{AsThread, UserThreadData},
};

use super::*;

pub fn open(address: Vaddr, flags: i32, mode: u32) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    let access_mode = AccessMode::try_from(flags)?;
    let open_flags = OpenFlags::from(flags);

    let _mode = InodeMode::from(mode);

    let mut path = Vec::new();
    loop {
        let byte = data
            .memory_info()
            .vm_space()
            .reader(address + path.len(), 1)
            .unwrap()
            .read_val()?;
        if byte == 0 {
            break;
        }
        path.push(byte);
    }

    let path = Path::new(core::str::from_utf8(&path).map_err(|_| SyscallError::InvalidArguments)?);

    if let Some(file) = open_file(&path) {
        let fd = data.fs_info().add_file(file, access_mode, open_flags);
        Ok(fd as isize)
    } else if open_flags.contains(OpenFlags::O_CREAT) {
        let parent = path.parent().ok_or(SyscallError::PermissionDenied)?;
        let file = open_file(&parent).ok_or(SyscallError::NotFound)?;

        let file = file
            .create(
                path.name(),
                if open_flags.contains(OpenFlags::O_DIRECTORY) {
                    FileType::Directory
                } else {
                    FileType::File
                },
            )
            .ok_or(SyscallError::InvalidArguments)?;

        let fd = data.fs_info().add_file(file, access_mode, open_flags);
        Ok(fd as isize)
    } else {
        Err(SyscallError::NotFound)
    }
}

pub fn close(fd: FileDescriptor) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    data.fs_info()
        .remove_file(fd)
        .ok_or(SyscallError::NotFound)?;
    Ok(0)
}

pub fn read(fd: FileDescriptor, address: Vaddr, len: usize) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    data.fs_info()
        .with_file_mut(fd, |offset, access_mode, _open_flags, file| {
            if !access_mode.is_readable() {
                return Err(SyscallError::PermissionDenied);
            };

            let mut buf = vec![0; len];
            let len = file.read_at(*offset, &mut buf);

            for (id, byte) in buf.iter().enumerate() {
                data.memory_info()
                    .vm_space()
                    .writer(address + id, 1)
                    .unwrap()
                    .write_val(byte)?;
            }

            *offset += len as u64;

            Ok(len as isize)
        })
        .ok_or(SyscallError::NotFound)?
}

pub fn write(fd: FileDescriptor, address: Vaddr, len: usize) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    data.fs_info()
        .with_file_mut(fd, |offset, access_mode, _open_flags, file| {
            if !access_mode.is_writable() {
                return Err(SyscallError::PermissionDenied);
            }

            let buffer = (0..len)
                .map(|id| {
                    data.memory_info()
                        .vm_space()
                        .reader(address + id, 1)
                        .unwrap()
                        .read_val::<u8>()
                })
                .collect::<Result<Vec<_>, _>>()?;

            let len = file.write_at(*offset, &buffer);
            *offset += len as u64;

            Ok(len as isize)
        })
        .ok_or(SyscallError::NotFound)?
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
                return Err(SyscallError::PermissionDenied);
            }

            let mut total_len = 0;

            for i in 0..count {
                let vec = data
                    .memory_info()
                    .vm_space()
                    .reader(iov_address + (i * 2 * 8), size_of::<IoVec>())
                    .unwrap()
                    .read_val::<IoVec>()?;

                let IoVec { base, len } = vec;

                if len > 0xffffff {
                    return Ok(0);
                }

                let buffer = (0..len)
                    .map(|id| {
                        data.memory_info()
                            .vm_space()
                            .reader(base + id, 1)
                            .unwrap()
                            .read_val::<u8>()
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                let len = file.write_at(*offset, &buffer);

                *offset += len as u64;
                total_len += len;
            }

            Ok(total_len as isize)
        })
        .ok_or(SyscallError::NotFound)?
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LseekWhence {
    Set = 0,
    Current = 1,
    End = 2,
}

impl LseekWhence {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(LseekWhence::Set),
            1 => Some(LseekWhence::Current),
            2 => Some(LseekWhence::End),
            _ => None,
        }
    }
}

impl From<LseekWhence> for i32 {
    fn from(value: LseekWhence) -> Self {
        value as i32
    }
}

pub fn lseek(fd: FileDescriptor, offset: isize, whence: LseekWhence) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    data.fs_info()
        .with_file_mut(fd, |offset_ref, _access_mode, _open_flags, file| {
            let file_type = file.r#type();

            if !file_type.seekable() {
                return Err(SyscallError::PermissionDenied);
            }

            match whence {
                LseekWhence::Set => *offset_ref = offset as u64,
                LseekWhence::Current => *offset_ref = (*offset_ref as i64 + offset as i64) as u64,
                LseekWhence::End => *offset_ref = (file.len() as i64 + offset as i64) as u64,
            }

            Ok(*offset_ref as isize)
        })
        .unwrap_or(Err(SyscallError::NotFound))
}

pub enum FcntlCommand {
    GetFd = 1,
    SetFd = 2,
}

impl FcntlCommand {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(FcntlCommand::GetFd),
            2 => Some(FcntlCommand::SetFd),
            _ => None,
        }
    }
}

impl From<FcntlCommand> for i32 {
    fn from(value: FcntlCommand) -> Self {
        value as i32
    }
}

pub fn fcntl(fd: FileDescriptor, cmd: FcntlCommand, _arg: u32) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    data.fs_info()
        .with_file_mut(fd, |_, _access_mode, open_flags, _file| match cmd {
            FcntlCommand::GetFd => Ok(open_flags.bits() as isize),
            FcntlCommand::SetFd => {
                *open_flags |= OpenFlags::O_CLOEXEC;
                Ok(0)
            }
        })
        .unwrap_or(Err(SyscallError::NotFound))
}

pub fn ioctl(fd: FileDescriptor, cmd: u32, arg: Vaddr) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    data.fs_info()
        .with_file_mut(fd, |_, _access_mode, _open_flags, file| {
            file.ioctl(cmd, arg).map(|res| res as isize)
        })
        .map(|val| val.map_err(|error| error.into()))
        .unwrap_or(Err(SyscallError::NotFound))
}
