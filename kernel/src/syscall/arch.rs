use zodiac::{mem::VirtualAddress, task::Task};

use crate::task::ThreadData;

use super::*;

#[repr(usize)]
#[derive(Debug)]
pub enum ArchPrctlOptions {
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

pub fn arch_prctl(options: ArchPrctlOptions, address: VirtualAddress) -> SyscallResult {
    let current_thread = Task::current();
    let data = current_thread.data().downcast_ref::<ThreadData>().unwrap();
    match options {
        ArchPrctlOptions::SetFs => data.fs.write().replace(address),
        ArchPrctlOptions::SetGs => data.gs.write().replace(address),
        ArchPrctlOptions::GetFs => return Ok(data.fs.read().unwrap_or(0) as isize),
        ArchPrctlOptions::GetGs => return Ok(data.gs.read().unwrap_or(0) as isize),
    };
    Ok(0)
}
