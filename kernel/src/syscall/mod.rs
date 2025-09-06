use alloc::vec;
use thiserror::Error;
use zodiac::{
    hal::{context::TrapFrame, trap::set_syscall_handler}, mem::VirtualAddress, ZodiacError
};

use crate::{filesystem::FileDescriptor, task::Process};

use arch::*;
use filesystem::*;
use mem::*;
use task::*;

mod arch;
mod filesystem;
mod mem;
mod task;

pub fn init() {
    set_syscall_handler(syscall_handler);
}

type SyscallResult = Result<isize, SyscallError>;

#[repr(isize)]
#[derive(Debug, Error)]
pub enum SyscallError {
    #[error("Invalid Arguments.")]
    InvalidArguments = -2,
    #[error("Permission denied.")]
    PermissionDenied = -3,
    #[error("Not found.")]
    NotFound = -4,
    #[error("Other.")]
    Other = i32::MIN as isize,
}

impl From<ZodiacError> for SyscallError {
    fn from(value: ZodiacError) -> Self {
        match value {
            ZodiacError::InvalidArguments => SyscallError::InvalidArguments,
            _ => SyscallError::Other,
        }
    }
}

fn syscall_handler(frame: &mut TrapFrame) -> isize {
    let syscall_id = frame.syscall_index();
    let [arg1, arg2, arg3, arg4, arg5, arg6] = frame.syscall_arguments();

    let mut matcher = || match syscall_id {
        0 => read(arg1 as FileDescriptor, arg2, arg3),
        1 => write(arg1 as FileDescriptor, arg2, arg3),
        2 => open(arg1, arg2 as i32, arg3 as u32),
        3 => close(arg1 as FileDescriptor),
        8 => lseek(
            arg1 as FileDescriptor,
            arg2 as isize,
            LseekWhence::from_i32(arg3 as i32).ok_or(SyscallError::InvalidArguments)?,
        ),
        9 => mmap(
            arg1,
            arg2,
            MMapProtection::from_bits_truncate(arg3 as i32),
            MMapFlags::from_bits_truncate(arg4 as i32),
            arg5,
            arg6,
        ),
        10 => mprotect(arg1, arg2, MMapProtection::from_bits_truncate(arg3 as i32)),
        11 => munmap(arg1, arg2),
        20 => {
            writev(arg1 as FileDescriptor, arg2 as VirtualAddress, arg3)
        },
        60 => exit(arg1 as i32),
        72 => fcntl(
            arg1 as FileDescriptor,
            FcntlCommand::from_i32(arg2 as i32).ok_or(SyscallError::InvalidArguments)?,
            arg3 as u32,
        ),
        158 => arch_prctl(ArchPrctlOptions::try_from(arg1)?, arg2),
        186 => get_tid(),
        218 => set_tid_address(arg1),
        231 => exit(arg1 as i32),
        _ => {
            log::warn!("Unimplemented syscall{}", syscall_id);
            Ok(0)
        } //_ => Err(SyscallError::SyscallNotSupported),
    };

    let result = matcher();

    let result = match result {
        Ok(value) => value,
        Err(error) => error as isize,
    };
    
    log::trace!(
        "syscall{}({:x}, {:x}, {:x}, {:x}, {:x}, {:x}) = {}",
        syscall_id,
        arg1,
        arg2,
        arg3,
        arg4,
        arg5,
        arg6,
        result,
    );
    result
}
