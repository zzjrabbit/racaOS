use crate::{
    fs::operation::{
        get_inode_by_fd, init_file_descriptor_manager,
        init_file_descriptor_manager_with_stdin_stdout,
    },
    user::get_current_process,
};
use alloc::vec;

use framework::{memory::addr_to_mut_ref, task::Process};
use x86_64::VirtAddr;

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct ProcessInfo {
    binary_addr: usize,
    binary_len: usize,
    name_addr: usize,
    name_len: usize,
    stdin: usize,
    stdout: usize,
}

pub fn create_process(info_addr: usize) -> usize {
    let info: &mut ProcessInfo = addr_to_mut_ref(VirtAddr::new(info_addr as u64));

    let binary_addr = info.binary_addr;
    let binary_len = info.binary_len;
    let name_addr = info.name_addr;
    let name_len = info.name_len;
    let stdin = info.stdin;
    let stdout = info.stdout;

    let func = || {
        let mut buf = vec![0; binary_len];

        if let Err(_) = get_current_process().read().page_table.read(
            VirtAddr::new(binary_addr as u64),
            binary_len,
            &mut buf,
        ) {
            panic!("Read error at {:x}!", binary_addr);
        }

        let mut name = vec![0; name_len];

        if let Err(_) = get_current_process().read().page_table.read(
            VirtAddr::new(name_addr as u64),
            name_len,
            &mut name,
        ) {
            panic!("Read error at {:x}!", name_addr);
        }

        let name = core::str::from_utf8(name.as_slice());

        if let Err(_) = name {
            return 0;
        }

        let process = Process::new_user_process(name.unwrap(), buf.leak());
        let pid = process.read().id;
        log::info!("tid: {}",process.read().threads.front().unwrap().read().id.0);

        //if let Some(stdin) = get_inode_by_fd(stdin) {
        //    let stdout = get_inode_by_fd(stdout).unwrap();
        //    init_file_descriptor_manager_with_stdin_stdout(pid, stdin, stdout);
        //} else {
            init_file_descriptor_manager(pid);
            //}
        log::info!("Ok");
        //x86_64::instructions::interrupts::enable();

        0
    };
    
    func()

    //x86_64::instructions::interrupts::without_interrupts(func)
}
