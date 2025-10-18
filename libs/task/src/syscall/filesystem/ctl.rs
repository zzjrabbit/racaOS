use ostd::{mm::Vaddr, task::Task};

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
        .unwrap_or(Err(Errno::EBADFD.no_message()))
}

pub fn ioctl(fd: FileDescriptor, cmd: u32, arg: Vaddr) -> SyscallResult {
    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();

    data.fs_info()
        .with_file_mut(fd, |_, _access_mode, _open_flags, file| {
            file.ioctl(cmd, arg).map(|res| res as isize)
        })
        .map(|val| val.map_err(|error| error.into()))
        .unwrap_or(Err(Errno::EBADFD.no_message()))
}
