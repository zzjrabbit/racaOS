use ::filesystem::IoctlCmd;
use alloc::sync::Arc;
use int_to_c_enum::TryFromInt;
use ostd::{mm::Vaddr, task::Task};

use crate::FileSystemInfo;

use {
    crate::{AsThread, UserThreadData},
    ::filesystem::{FileDescriptor, OpenFlags},
};

use super::*;

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
                return Err(Errno::EACCES.no_message());
            }

            match whence {
                LseekWhence::Set => *offset_ref = offset as u64,
                LseekWhence::Current => *offset_ref = (*offset_ref as i64 + offset as i64) as u64,
                LseekWhence::End => *offset_ref = (file.len() as i64 + offset as i64) as u64,
            }

            Ok(*offset_ref as isize)
        })
        .unwrap_or(Err(Errno::EBADF.no_message()))
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, TryFromInt)]
pub enum FcntlCmd {
    DupFd = 0,
    GetFd = 1,
    SetFd = 2,
    GetFl = 3,
    SetFl = 4,
    GetLk = 5,
    SetLk = 6,
    SetLkw = 7,
    SetOwn = 8,
    GetOwn = 9,
    DupFdCloexec = 1030,
}

pub fn fcntl(fd: FileDescriptor, cmd: FcntlCmd, arg: u64) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();
    let fs_info = data.fs_info();

    return match cmd {
        FcntlCmd::GetFd => Ok(fs_info.get_open_flags(fd)?.bits() as isize),
        FcntlCmd::SetFd => {
            fs_info.with_open_flags_mut(fd, |open_flags| {
                *open_flags |= OpenFlags::O_CLOEXEC;
            })?;
            Ok(0)
        }
        FcntlCmd::DupFd => handle_dupfd(fs_info, fd, arg, OpenFlags::empty()),
        FcntlCmd::DupFdCloexec => handle_dupfd(fs_info, fd, arg, OpenFlags::O_CLOEXEC),
        _ => unimplemented!(),
    };

    fn handle_dupfd(
        fs_info: &Arc<FileSystemInfo>,
        fd: FileDescriptor,
        arg: u64,
        flags: OpenFlags,
    ) -> SyscallResult {
        fs_info
            .duplicate(fd, arg as FileDescriptor, flags)
            .map(|fd| fd as isize)
    }
}

pub fn ioctl(fd: FileDescriptor, cmd: u32, arg: Vaddr) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    data.fs_info()
        .with_file_mut(fd, |_, _access_mode, _open_flags, file| {
            file.ioctl(
                data.memory_info().vmar(),
                IoctlCmd::try_from(cmd).map_err(|_| Errno::EINVAL.no_message())?,
                arg,
            )
            .map(|res| res as isize)
        })
        .map(|val| val.map_err(|error| error.into()))
        .unwrap_or(Err(Errno::EBADFD.no_message()))
}
