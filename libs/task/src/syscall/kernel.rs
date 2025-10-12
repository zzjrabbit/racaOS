use ostd::{mm::Vaddr, task::Task, Pod};

use crate::{
    syscall::SyscallResult,
    AsThread, UserThreadData,
};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct UtsName {
    sysname: [u8; 65],
    nodename: [u8; 65],
    release: [u8; 65],
    version: [u8; 65],
    machine: [u8; 65],
    domainname: [u8; 65],
}

#[allow(unsafe_code)]
unsafe impl Pod for UtsName {}

impl UtsName {
    pub fn new() -> Self {
        UtsName {
            sysname: [0; 65],
            nodename: [0; 65],
            release: [0; 65],
            version: [0; 65],
            machine: [0; 65],
            domainname: [0; 65],
        }
    }
}

pub fn uname(address: Vaddr) -> SyscallResult {
    let task = Task::current().unwrap();
    let data = task.direct_downcast::<UserThreadData>().unwrap();

    let mut utsname = UtsName::new();
    utsname.sysname.copy_from_slice(b"racaOS");
    utsname.release.copy_from_slice(b"RELEASE");
    utsname
        .version
        .copy_from_slice(core::env!("CARGO_PKG_VERSION").as_bytes());
    utsname.machine.copy_from_slice(b"x86_64");
    utsname.nodename.copy_from_slice(b"root");

    data.memory_info().vmar().write_val(address, &utsname)?;

    Ok(0)
}
