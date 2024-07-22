use crate::{fs::operation::init_file_descriptor_manager, user::get_current_process};
use alloc::vec;

use framework::task::Process;
use x86_64::VirtAddr;

pub fn create_process(buf_addr: usize, buf_len: usize, name_addr: usize, name_len: usize) -> usize {
    let func = || {
        let mut buf = vec![0; buf_len];

        if let Err(_) = get_current_process().read().page_table.read(
            VirtAddr::new(buf_addr as u64),
            buf_len,
            &mut buf,
        ) {
            panic!("Read error at {:x}!", buf_addr);
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
        init_file_descriptor_manager(process.read().id);

        0
    };

    x86_64::instructions::interrupts::without_interrupts(func)
}
