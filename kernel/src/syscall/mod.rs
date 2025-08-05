use alloc::{string::ToString, vec};
use thiserror::Error;
use zodiac::{
    ZodiacError,
    hal::trap::set_syscall_handler,
};

use crate::task::Process;

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
    #[error("Syscall not supported.")]
    SyscallNotSupported = -1,
    #[error("Invalid Arguments.")]
    InvalidArguments = -2,
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

fn syscall_handler(
    syscall_id: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) -> isize {
    log::trace!(
        "syscall{}({:x}, {:x}, {:x}, {:x}, {:x}, {:x})",
        syscall_id,
        arg1,
        arg2,
        arg3,
        arg4,
        arg5,
        arg6
    );

    let matcher = || match syscall_id {
        1 => write(arg1, arg2, arg3),
        9 => mmap(arg1, arg2, arg3, arg4, arg5, arg6),
        60 => exit(arg1 as i32),
        158 => arch_prctl(ArchPrctlOptions::try_from(arg1)?, arg2),
        218 => set_tid_address(arg1),
        _ => Ok(0)
        //_ => Err(SyscallError::SyscallNotSupported),
    };

    let result = matcher();

    match result {
        Ok(value) => value,
        Err(error) => error as isize,
    }
}
