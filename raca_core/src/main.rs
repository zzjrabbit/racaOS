#![no_std]
#![no_main]

use core::panic::PanicInfo;
use raca_core::device::keyboard::print_keypresses;
use raca_core::device::rtc::RtcDateTime;
use raca_core::task::{Process, Thread};
use limine::BaseRevision;

#[used]
#[link_section = ".requests"]
static BASE_REVISION: BaseRevision = BaseRevision::with_revision(1);

#[no_mangle]
extern "C" fn _start() -> ! {
    raca_core::init();
    Thread::new_kernel_thread(print_keypresses);
    let current_time = RtcDateTime::new().to_datetime().unwrap();
    log::info!("Current time: {}", current_time);

    let hello_raw_elf = include_bytes!("../../apps/hello1.rae");
    Process::new_user_process("Hello", hello_raw_elf).unwrap();

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(panic_info: &PanicInfo<'_>) -> ! {
    log::error!("{}", panic_info);
    loop {
        x86_64::instructions::hlt();
    }
}
