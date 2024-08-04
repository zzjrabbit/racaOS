use core::alloc::Layout;

use crate::{
    fs::operation::{
        get_inode_by_fd, init_file_descriptor_manager,
        init_file_descriptor_manager_with_stdin_stdout,
    },
    user::{get_current_process, get_current_thread},
};
use alloc::{sync::Arc, vec};

use framework::{
    memory::addr_to_mut_ref,
    task::{signal::Signal, thread::ThreadState, Process},
};
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

        if let Some(stdin) = get_inode_by_fd(stdin) {
            let stdout = get_inode_by_fd(stdout).unwrap();
            init_file_descriptor_manager_with_stdin_stdout(pid, stdin, stdout);
        } else {
            init_file_descriptor_manager(pid);
        }

        process.write().father = Some(Arc::downgrade(&get_current_process()));

        pid.0 as usize
    };

    func()

    //x86_64::instructions::interrupts::without_interrupts(func)
}

pub fn exit(code: usize) -> usize {
    let process = get_current_process();
    if let Some(ref father) = process.read().father {
        father
            .upgrade()
            .unwrap()
            .write()
            .signal_manager
            .register_signal(
                0,
                Signal {
                    ty: 0,
                    data: [code as u64, 0, 0, 0, 0, 0, 0, 0],
                },
            );
    }
    framework::task::scheduler::exit();
    return 0;
}

pub fn has_signal(ty: usize) -> usize {
    let process = get_current_process();
    let process = process.read();
    process.signal_manager.has_signal(ty) as usize
}

pub fn start_wait_for_signal(ty: usize) -> usize {
    let process = get_current_process();
    process.write().signal_manager.register_wait_for(ty);
    get_current_thread().write().state = ThreadState::Waiting;
    return 0;
}

pub fn get_signal(ty: usize) -> usize {
    let process = get_current_process();
    let mut process = process.write();
    if let Some(signal) = process.signal_manager.get_signal(ty) {
        let new_signal_address = process.heap.allocate(Layout::from_size_align(size_of::<Signal>(), 8).unwrap()).unwrap();
        let new_signal = addr_to_mut_ref(VirtAddr::new(new_signal_address));
        *new_signal = signal;
        new_signal_address as usize
    } else {
        0
    }
}

pub fn done_signal(ty: usize) -> usize {
    let process = get_current_process();
    process.write().signal_manager.delete_signal(ty);
    0
}
