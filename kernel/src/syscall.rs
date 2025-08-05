use alloc::{string::ToString, vec};
use thiserror::Error;
use zodiac::{
    ZodiacError,
    hal::trap::set_syscall_handler,
    mem::{MMUFlags, PhysicalMemoryAllocOptions, VirtualAddress},
    task::Thread,
};

use crate::task::Process;

pub fn init() {
    set_syscall_handler(syscall_handler);
}

type SyscallResult = Result<isize, SyscallError>;

#[repr(isize)]
#[derive(Debug, Error)]
enum SyscallError {
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
        158 => arch_prctl(ArchPrctlOptions::try_from(arg1)?, arg2),
        _ => Err(SyscallError::SyscallNotSupported),
    };

    let result = matcher();

    match result {
        Ok(value) => value,
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
        ArchPrctlOptions::GetFs => return Ok(current_thread.fs_base() as isize),
        ArchPrctlOptions::GetGs => return Ok(current_thread.gs_base() as isize),
    }
    Ok(0)
}

fn mmap(
    address: usize,
    len: usize,
    _protection: usize,
    _flags: usize,
    _fd: usize,
    _offset: usize,
) -> SyscallResult {
    let current_process = Process::current();

    let (address, mut cursor, page_size) = if address == 0 {
        current_process.allocate(len, false)
    } else {
        current_process.allocate_at(address, len, false)
    }?;

    let physical_memory = PhysicalMemoryAllocOptions::default()
        .count(
            page_size.align_up(len + address - page_size.align_down(address)) / page_size as usize,
        )
        .page_size(page_size)
        .allocate()
        .unwrap();

    cursor.map(&physical_memory, MMUFlags::USER_CODE | MMUFlags::USER_DATA)?;

    Ok(address as isize)
}

fn write(_fd: usize, address: VirtualAddress, len: usize) -> SyscallResult {
    let current_process = Process::current();

    let mut buffer = vec![0; len];
    current_process
        .inner()
        .vm_space()
        .reader(address, len)
        .read(&mut buffer)?;

    log::trace!("Writing buffer: {:?}", buffer);

    crate::terminal::terminal_write(
        core::str::from_utf8(&buffer)
            .map_err(|_| SyscallError::InvalidArguments)?
            .to_string(),
    );

    Ok(address as isize)
}
