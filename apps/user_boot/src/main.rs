#![no_std]
#![no_main]

use common_std::{
    ARG_HANDLE,
    handle::close_handle,
    ipc::{Channel, MessagePacket},
    signal::{Signal, wait_for_signal},
    task::{BasicPolicy, Job, PolicyAction, PolicyCondition},
};

use alloc::{format, vec};

extern crate alloc;

#[unsafe(no_mangle)]
pub unsafe extern "sysv64" fn _start() -> ! {
    let root_job = unsafe { Job::from_handle(ARG_HANDLE) };

    let system_job = root_job.create_child().unwrap();
    let user_job = root_job.create_child().unwrap();

    user_job
        .set_basic_policy(&[BasicPolicy {
            condition: PolicyCondition::NewDdkObject,
            action: PolicyAction::Deny,
        }])
        .unwrap();

    let (channel0, channel1) = Channel::create().unwrap();

    common_std::task::Process::create(
        &system_job,
        "terminal",
        include_bytes!(core::env!("CARGO_BIN_FILE_TERMINAL")),
        vec![channel1.as_handle()],
    )
    .unwrap();

    let (channel2, channel3) = Channel::create().unwrap();
    channel0
        .write(&MessagePacket::new(vec![], vec![channel3.as_handle()]))
        .unwrap();

    common_std::task::Process::create(
        &system_job,
        "keyboard",
        include_bytes!(core::env!("CARGO_BIN_FILE_KEYBOARD")),
        vec![channel2.as_handle()],
    )
    .unwrap();

    let (channel2, channel3) = Channel::create().unwrap();
    channel0
        .write(&MessagePacket::new(vec![], vec![channel3.as_handle()]))
        .unwrap();

    let rash = common_std::task::Process::create(
        &user_job,
        "rash",
        include_bytes!(core::env!("CARGO_BIN_FILE_RASH")),
        vec![channel2.as_handle()],
    )
    .unwrap();

    rash.wait().unwrap();
    let info = rash.get_info().unwrap();
    common_std::debug::debug(&format!("rash exited with code {} :(\n", info.return_code)).unwrap();
    close_handle(rash.as_handle()).unwrap();

    loop {}
}

#[panic_handler]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    common_std::debug::debug("panic").unwrap();
    loop {}
}
