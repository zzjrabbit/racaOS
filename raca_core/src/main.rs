#![no_std]
#![no_main]

use framework::{
    init_framework,
    task::{Process, Thread},
    user::regist_syscall_handler,
};
use limine::BaseRevision;
use raca_core::{
    fs::{self, operation::init_file_descriptor_manager},
    user::syscall::syscall_handler,
};

extern crate alloc;

#[used]
#[link_section = ".requests"]
static BASE_REVISION: BaseRevision = BaseRevision::with_revision(1);

pub mod drivers;

#[no_mangle]
pub extern "C" fn _start() {
    init_framework();
    raca_core::drivers::xhci::init();
    fs::init();
    raca_core::ui::init();

    regist_syscall_handler(syscall_handler);

    let hello1 = raca_core::fs::operation::kernel_open("/RACA/app64/init.rae".into()).unwrap();
    let size = hello1.read().size();
    let buf = alloc::vec![0; size].leak();
    hello1.read().read_at(0, buf);

    Thread::new_kernel_thread(raca_core::fs::vfs::dev::terminal::keyboard_parse_thread);

    let process = Process::new_user_process("init", buf);
    init_file_descriptor_manager(process.read().id);
    //Process::new_user_process("Hello2", include_bytes!("../../../apps/hello2.rae"));

    (40..=47).for_each(|index| framework::print!("\x1b[{}m   \x1b[0m", index));
    framework::println!();
    (100..=107).for_each(|index| framework::print!("\x1b[{}m   \x1b[0m", index));
    framework::println!();

    (40..=47).for_each(|index| framework::serial_print!("\x1b[{}m   \x1b[0m", index));
    framework::serial_println!();
    (100..=107).for_each(|index| framework::serial_print!("\x1b[{}m   \x1b[0m", index));
    framework::serial_println!();

    framework::start_schedule();
    loop {}
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("{}", info);
    loop {}
}
