use ostd::{Pod, mm::Vaddr, task::Task};

use crate::{AsThread, UserThreadData, syscall::SyscallResult};

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
    
    let sys_name = b"racaOS";
    utsname.sysname[0..sys_name.len()].copy_from_slice(sys_name);
    
    let release = b"RELEASE";
    utsname.release[0..release.len()].copy_from_slice(release);
    
    let version = core::env!("CARGO_PKG_VERSION").as_bytes();
    utsname.version[0..version.len()].copy_from_slice(version);
    
    let machine = b"x86_64";
    utsname.machine[0..machine.len()].copy_from_slice(machine);
    
    let domainname = b"domain";
    utsname.domainname[0..domainname.len()].copy_from_slice(domainname);
    
    let nodename = b"root";
    utsname.nodename[0..nodename.len()].copy_from_slice(nodename);
    
    data.memory_info().vmar().write_val(address, &utsname)?;

    Ok(0)
}
