use errors::Error;
use ostd::{mm::Vaddr, prelude::println};

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
    type Error = Error;

    fn try_from(value: usize) -> Result<Self> {
        match value {
            Self::SET_FS => Ok(Self::SetFs),
            Self::GET_FS => Ok(Self::GetFs),
            Self::SET_GS => Ok(Self::SetGs),
            Self::GET_GS => Ok(Self::GetGs),
            _ => Err(Errno::EINVAL.no_message()),
        }
    }
}

pub fn arch_prctl(
    options: ArchPrctlOptions,
    address: Vaddr,
    user_context: &mut UserContext,
) -> SyscallResult {
    match options {
        ArchPrctlOptions::SetFs => {
            user_context.set_tls_pointer(address);
            user_context.activate_tls_pointer();
        }
        ArchPrctlOptions::GetFs => return Ok(user_context.tls_pointer() as isize),
        _ => {
            println!("Users shouldn't access gs.");
            return Err(Errno::EACCES.no_message());
        }
    };
    Ok(0)
}
