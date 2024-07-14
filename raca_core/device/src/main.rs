#![no_std]
#![no_main]

use alloc::vec;
use device::fs;
use framework::{
    arch::apic::get_lapic_id,
    init_framework,
    task::{scheduler::SCHEDULERS, Process},
    user::regist_syscall_handler,
};
use limine::BaseRevision;
use x86_64::VirtAddr;

extern crate alloc;

#[used]
#[link_section = ".requests"]
static BASE_REVISION: BaseRevision = BaseRevision::with_revision(1);

pub mod drivers;

#[no_mangle]
pub extern "C" fn _start() {
    init_framework();
    fs::init();

    regist_syscall_handler(syscall_handler);
    Process::new_user_process("Hello1", include_bytes!("../../../apps/hello1.rae"));
    Process::new_user_process("Hello2", include_bytes!("../../../apps/hello2.rae"));

    //let mut buf = [0u8; 512];
    //drivers::ahci::read_block(0, 0, &mut buf).unwrap();
    //framework::serial_println!("Hello, Frame Kernel! {:?}", buf);

    //framework::serial_println!("{:?}",crate::drivers::ahci::DISK_START);

    framework::start_schedule();
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

            if let Err(_) = SCHEDULERS
                .lock()
                .get(&get_lapic_id())
                .unwrap()
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

            framework::print!("{}", core::str::from_utf8(buf.as_slice()).unwrap());
        }
        1 => {
            framework::print!("[{}]", framework::arch::apic::get_lapic_id());
            // 在这里输出当前线程所在的CPU的lapic id
        }
        _ => {}
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("{}", info);
    loop {}
}
