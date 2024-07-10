#![no_std]
#![no_main]

use alloc::vec;
use framework::{
    init_framework, println,
    task::{scheduler::SCHEDULER, Process},
    user::regist_syscall_handler,
};
use limine::BaseRevision;
use x86_64::VirtAddr;

extern crate alloc;

#[used]
#[link_section = ".requests"]
static BASE_REVISION: BaseRevision = BaseRevision::with_revision(1);

#[no_mangle]
pub extern "C" fn _start() {
    init_framework();
    regist_syscall_handler(syscall_handler);
    //Process::new_user_process("Hello1", include_bytes!("../../../apps/hello1.rae"));
    println!("Hello, Frame Kernel!");
    x86_64::instructions::interrupts::enable();
    loop {}
}

#[allow(unused_variables)]
pub fn syscall_handler(
    idx: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) {
    match idx {
        0 => {
            let buf_addr = arg1;
            let buf_len = arg2;
            let mut buf = vec![0; buf_len];

            framework::println!("read : {:x}", buf_addr);

            if let Err(_) = SCHEDULER
                .read()
                .current_thread
                .read()
                .process
                .upgrade()
                .unwrap()
                .read()
                .page_table
                .read(VirtAddr::new(buf_addr as u64), buf_len, &mut buf)
            {
                panic!("Read error at {:x}!", buf_addr);
            }

            framework::println!("{}", core::str::from_utf8(buf.as_slice()).unwrap());
        }
        _ => {}
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("{}", info);
    loop {}
}
