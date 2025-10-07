use alloc::vec;
use ostd::{arch::cpu::context::UserContext, mm::Vaddr, Error as OstdError};
use thiserror::Error;

use crate::{filesystem::FileDescriptor, task::Process};

use arch::*;
use filesystem::*;
use kernel::*;
use mem::*;
use task::*;

mod arch;
mod filesystem;
mod kernel;
mod mem;
mod task;

pub fn init() {}

type SyscallResult = Result<isize, SyscallError>;

#[repr(isize)]
#[derive(Debug, Error)]
pub enum SyscallError {
    #[error("Null")]
    Null = 0,
    #[error("Invalid Arguments.")]
    InvalidArguments = -2,
    #[error("Permission denied.")]
    PermissionDenied = -3,
    #[error("Not found.")]
    NotFound = -4,
    #[error("Buffer too small.")]
    BufferTooSmall = -5,
    #[error("Other.")]
    Other = i32::MIN as isize,
}

impl From<OstdError> for SyscallError {
    fn from(error: OstdError) -> Self {
        match error {
            OstdError::AccessDenied => SyscallError::PermissionDenied,
            OstdError::InvalidArgs => SyscallError::InvalidArguments,
            _ => SyscallError::Other,
        }
    }
}

pub fn syscall_handler(context: &mut UserContext) {
    let syscall_id = context.rax();
    let [arg1, arg2, arg3, arg4, arg5, arg6] = [
        context.rdi(),
        context.rsi(),
        context.rdx(),
        context.r10(),
        context.r8(),
        context.r9(),
    ];

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
        16 => ioctl(arg1 as FileDescriptor, arg2 as u32, arg3 as Vaddr),
        20 => writev(arg1 as FileDescriptor, arg2 as Vaddr, arg3),
        60 => exit(arg1 as i32),
        63 => uname(arg1 as Vaddr),
        72 => fcntl(
            arg1 as FileDescriptor,
            FcntlCommand::from_i32(arg2 as i32).ok_or(SyscallError::InvalidArguments)?,
            arg3 as u32,
        ),
        79 => getcwd(arg1 as Vaddr, arg2),
        80 => chdir(arg1 as Vaddr),
        81 => fchdir(arg1 as FileDescriptor),
        158 => arch_prctl(ArchPrctlOptions::try_from(arg1)?, arg2, context),
        186 => get_tid(),
        218 => set_tid_address(arg1),
        231 => exit(arg1 as i32),
        _ => {
            log::warn!(target: "kernel", "Unimplemented syscall{}", syscall_id);
            Ok(0)
        } //_ => Err(SyscallError::SyscallNotSupported),
    };

    let result = matcher();

    let result = match result {
        Ok(value) => value,
        Err(error) => error as isize,
    };

    log::info!(
        target: "kernel",
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
    context.set_rax(result as usize);
}
