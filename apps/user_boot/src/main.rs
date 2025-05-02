#![no_std]
#![no_main]

use common_std::ipc::{Channel, MessagePacket};

use alloc::vec;

extern crate alloc;

#[unsafe(no_mangle)]
pub unsafe extern "sysv64" fn _start() -> ! {
    let (channel0, channel1) = Channel::create().unwrap();

    common_std::task::Process::create(
        "terminal",
        include_bytes!(core::env!("CARGO_BIN_FILE_TERMINAL")),
        vec![channel1.as_handle()],
    )
    .unwrap();

    print_to_terminal(&channel0, "Hello World From User Boot!\n");

    let (channel2, channel3) = Channel::create().unwrap();
    channel0
        .write(&MessagePacket::new(vec![], vec![channel3.as_handle()]))
        .unwrap();

    common_std::task::Process::create(
        "keyboard",
        include_bytes!(core::env!("CARGO_BIN_FILE_KEYBOARD")),
        vec![channel2.as_handle()],
    )
    .unwrap();

    let (channel2, channel3) = Channel::create().unwrap();
    channel0
        .write(&MessagePacket::new(vec![], vec![channel3.as_handle()]))
        .unwrap();

    common_std::task::Process::create(
        "rash",
        include_bytes!(core::env!("CARGO_BIN_FILE_RASH")),
        vec![channel2.as_handle()],
    )
    .unwrap();

    loop {}
}

fn print_to_terminal(channel: &Channel, msg: &str) {
    let mut data = alloc::vec![0];
    data.extend_from_slice(msg.as_bytes());
    channel
        .write(&MessagePacket::new(data, alloc::vec![]))
        .unwrap();
}
