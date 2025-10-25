use core::time::Duration;

use alloc::vec;
use errors::{Errno, Result};
use ostd::{Pod, arch::cpu::context::UserContext, mm::Vaddr};

use {crate::Process, ::filesystem::FileDescriptor};

use arch::*;
use filesystem::*;
use kernel::*;
use mem::*;
use signal::*;
use task::*;
use time::*;

mod arch;
mod filesystem;
mod kernel;
mod mem;
mod signal;
mod task;
mod time;

type SyscallResult = Result<isize>;

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
        4 => stat(arg1 as Vaddr, arg2 as Vaddr),
        5 => fstat(arg1 as FileDescriptor, arg2 as Vaddr),
        6 => lstat(arg1 as Vaddr, arg2 as Vaddr),
        8 => lseek(
            arg1 as FileDescriptor,
            arg2 as isize,
            LseekWhence::from_i32(arg3 as i32)
                .ok_or(Errno::EINVAL.with_message("Invalid whence."))?,
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
        13 => rt_sigaction(arg1 as u8, arg2 as Vaddr, arg3 as Vaddr, arg4 as u64),
        14 => rt_sigprocmask(arg1 as u32, arg2 as Vaddr, arg3 as Vaddr, arg4 as u64),
        16 => ioctl(arg1 as FileDescriptor, arg2 as u32, arg3 as Vaddr),
        17 => pread64(
            arg1 as FileDescriptor,
            arg2 as Vaddr,
            arg3 as usize,
            arg4 as u64,
        ),
        18 => pwrite64(
            arg1 as FileDescriptor,
            arg2 as Vaddr,
            arg3 as usize,
            arg4 as u64,
        ),
        19 => readv(arg1 as FileDescriptor, arg2 as Vaddr, arg3),
        20 => writev(arg1 as FileDescriptor, arg2 as Vaddr, arg3),
        24 => sched_yield(),
        39 => getpid(),
        57 => fork(context),
        59 => execve(arg1 as Vaddr, arg2 as Vaddr, arg3 as Vaddr, context),
        60 => exit(arg1 as i32),
        61 => wait4(arg1 as u64, arg2 as Vaddr, arg3 as u32, arg4 as Vaddr),
        63 => uname(arg1 as Vaddr),
        72 => fcntl(
            arg1 as FileDescriptor,
            FcntlCommand::from_i32(arg2 as i32)
                .ok_or(Errno::EINVAL.with_message("Invalid command."))?,
            arg3 as u32,
        ),
        79 => getcwd(arg1 as Vaddr, arg2),
        80 => chdir(arg1 as Vaddr),
        81 => fchdir(arg1 as FileDescriptor),
        102 => getuid(),
        110 => getppid(),
        158 => arch_prctl(ArchPrctlOptions::try_from(arg1)?, arg2, context),
        161 => chroot(arg1 as Vaddr),
        186 => get_tid(),
        218 => set_tid_address(arg1),
        228 => clock_gettime(arg1 as i32, arg2 as Vaddr),
        231 => exit(arg1 as i32),
        262 => fstatat(
            Some(arg1 as FileDescriptor),
            arg2 as Vaddr,
            arg3 as Vaddr,
            arg4 as u32,
        ),
        _ => {
            log::warn!("Unimplemented syscall{}", syscall_id);
            Ok(0)
        } //_ => Err(SyscallError::SyscallNotSupported),
    };

    let result = matcher();

    let result = match result {
        Ok(value) => value,
        Err(error) => -i32::from(error) as isize,
    };

    let pid = Process::current().id();

    log::info!(
        "[{}]syscall{}({:x}, {:x}, {:x}, {:x}, {:x}, {:x}) = {}",
        pid,
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

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, Default, Clone, Copy, Pod)]
struct timespec_t {
    pub sec: i64,
    pub nsec: i64,
}

impl From<Duration> for timespec_t {
    fn from(value: Duration) -> Self {
        let sec = value.as_secs() as i64;
        let nsec = value.subsec_nanos() as i64;
        Self { sec, nsec }
    }
}
