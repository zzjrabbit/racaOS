use zodiac::{hal::trap::set_syscall_handler, mem::VirtualAddress, task::Thread};
use thiserror::Error;

pub fn init() {
    set_syscall_handler(syscall_handler);
}

type SyscallResult = Result<usize, SyscallError>;

#[repr(isize)]
#[derive(Debug, Error)]
enum SyscallError {
    #[error("Syscall not supported.")]
    SyscallNotSupported = -1,
    #[error("Invalid Arguments.")]
    InvalidArguments = -2,
}


fn syscall_handler(syscall_id: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize, arg5: usize, arg6: usize) -> isize {
    log::trace!("syscall{}({:x}, {:x}, {:x}, {:x}, {:x}, {:x})", syscall_id, arg1, arg2, arg3, arg4, arg5, arg6);
    
    let matcher = || match syscall_id {
        158 => arch_prctl(ArchPrctlOptions::try_from(arg1)?, arg2),
        //_ => Err(SyscallError::SyscallNotSupported),
        _ => Ok(0),
    };

    let result = matcher();

    match result {
        Ok(value) => value as isize,
        Err(error) => error as isize,
    }
}

#[repr(usize)]
#[derive(Debug)]
enum ArchPrctlOptions {
    SetFs = Self::SET_FS,
    GetFs = Self::GET_FS,
    SetGs = Self::SET_GS,
    GetGs = Self::GET_GS,
}

impl ArchPrctlOptions {
    const SET_FS: usize = 0x1002;
    const GET_FS: usize = 0x1003;
    const SET_GS: usize = 0x1004;
    const GET_GS: usize = 0x1005;
}

impl TryFrom<usize> for ArchPrctlOptions {
    type Error = SyscallError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            Self::SET_FS => Ok(Self::SetFs),
            Self::GET_FS => Ok(Self::GetFs),
            Self::SET_GS => Ok(Self::SetGs),
            Self::GET_GS => Ok(Self::GetGs),
            _ => Err(Self::Error::InvalidArguments),
        }
    }
}

fn arch_prctl(options: ArchPrctlOptions, address: VirtualAddress) -> SyscallResult {
    log::trace!("arch_prctl({:?}, {:x})", options, address);

    let current_thread = Thread::current();
    match options {
        ArchPrctlOptions::SetFs => current_thread.set_fs_base(address),
        ArchPrctlOptions::SetGs => current_thread.set_gs_base(address),
        ArchPrctlOptions::GetFs => return Ok(current_thread.fs_base()),
        ArchPrctlOptions::GetGs => return Ok(current_thread.gs_base()),
    }
    Ok(0)
}

