use alloc::vec::Vec;
use zodiac::{mem::VirtualAddress, task::Task};

use crate::{
    filesystem::{AccessMode, FileDescriptor, FileType, InodeMode, OpenFlags, Path, open_file},
    task::ThreadData,
};

use super::*;

pub fn open(address: VirtualAddress, flags: i32, mode: u32) -> SyscallResult {
    let thread = Task::current();
    let data = thread.data().downcast_ref::<ThreadData>().unwrap();

    let access_mode = AccessMode::try_from(flags)?;
    let open_flags = OpenFlags::from(flags);

    let _mode = InodeMode::from(mode);

    let mut path = Vec::new();
    loop {
        let mut buffer = [0; 1];
        data.vm_space
            .reader(address + path.len(), 1)
            .read(&mut buffer)?;
        if buffer[0] == 0 {
            break;
        }
        path.push(buffer[0]);
    }

    let path = Path::new(Path::new(
        core::str::from_utf8(&path).map_err(|_| SyscallError::InvalidArguments)?,
    ));

    log::info!(
        "Opening {} with access {:?} flags {:?}",
        path,
        access_mode,
        open_flags
    );

    if let Some(file) = open_file(&path) {
        let fd = data.add_file(file, access_mode, open_flags);
        log::info!("fd: {}", fd);
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

            let fd = data.add_file(file, access_mode, open_flags);
            Ok(fd as isize)
        } else {
            Err(SyscallError::NotFound)
        }
}

pub fn close(fd: FileDescriptor) -> SyscallResult {
    let thread = Task::current();
    let data = thread.data().downcast_ref::<ThreadData>().unwrap();

    data.remove_file(fd).ok_or(SyscallError::NotFound)?;
    Ok(0)
}

pub fn read(fd: FileDescriptor, address: VirtualAddress, len: usize) -> SyscallResult {
    let thread = Task::current();
    let data = thread.data().downcast_ref::<ThreadData>().unwrap();

    data.with_file_mut(fd, |offset, access_mode, _open_flags, file| {
        if !access_mode.is_readable() {
            return Err(SyscallError::PermissionDenied);
        };

        let mut buf = vec![0; len];
        let len = file.read_at(*offset, &mut buf);

        data.vm_space.writer(address, len).write(&buf[0..len])?;

        *offset += len as u64;

        Ok(len as isize)
    })
    .ok_or(ZodiacError::NotFound)?
}

pub fn write(fd: FileDescriptor, address: VirtualAddress, len: usize) -> SyscallResult {
    let thread = Task::current();
    let data = thread.data().downcast_ref::<ThreadData>().unwrap();

    data.with_file_mut(fd, |offset, access_mode, _open_flags, file| {
        if !access_mode.is_writable() {
            return Err(SyscallError::PermissionDenied);
        }

        let mut buffer = vec![0; len];

        data.vm_space.reader(address, len).read(&mut buffer)?;

        log::info!("writing {:?}", buffer);

        let len = file.write_at(*offset, &buffer);
        *offset += len as u64;

        Ok(len as isize)
    })
    .ok_or(ZodiacError::NotFound)?
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
    let thread = Task::current();
    let data = thread.data().downcast_ref::<ThreadData>().unwrap();

    data.with_file_mut(fd, |offset_ref, _access_mode, _open_flags, file| {
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
    let thread = Task::current();
    let data = thread.data().downcast_ref::<ThreadData>().unwrap();

    data.with_file_mut(fd, |_, _access_mode, open_flags, _file| {
        match cmd {
            FcntlCommand::GetFd => Ok(open_flags.bits() as isize),
            FcntlCommand::SetFd => {
                *open_flags |= OpenFlags::O_CLOEXEC;
                Ok(open_flags.bits() as isize)
            }
        }
    })
    .unwrap_or(Err(SyscallError::NotFound))
}
