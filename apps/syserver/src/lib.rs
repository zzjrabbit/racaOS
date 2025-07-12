#![no_std]

use alloc::vec::Vec;
use common_std::ipc::{Channel, MessagePacket};
use common_std::signal::{wait_for_signal, Signal};
pub use common_std::*;
use spin::Mutex;

extern crate alloc;

unsafe extern "Rust" {
    fn main(handles: Vec<u32>);
}

static STDIO: Mutex<Option<Channel>> = Mutex::new(None);

#[unsafe(no_mangle)]
pub unsafe extern "sysv64" fn _start() -> ! {
    let mut handles = Vec::new();

    let channel_with_father = Channel::with_father();

    wait_for_signal(&[channel_with_father.as_handle()], Signal::READABLE).unwrap();

    let msg = channel_with_father.read().unwrap();
    let data = [
        msg.data[0],
        msg.data[1],
        msg.data[2],
        msg.data[3],
        msg.data[4],
        msg.data[5],
        msg.data[6],
        msg.data[7],
    ];
    let handle_num = usize::from_le_bytes(data);

    while handles.len() < handle_num {
        wait_for_signal(&[channel_with_father.as_handle()], Signal::READABLE).unwrap();

        let msg = channel_with_father.read().unwrap();
        for handle in msg.handles {
            handles.push(handle);
        }
    }

    let stdio = unsafe { Channel::from_handle(handles.pop().unwrap()) };
    *STDIO.lock() = Some(stdio);

    unsafe {
        main(handles);
    }

    loop {}
}

pub fn print(msg: &str) -> RcResult<()> {
    if let Some(ref mut stdio) = *STDIO.lock() {
        stdio.write(&MessagePacket::new(msg.as_bytes().to_vec(), Vec::new()))?;
        Ok(())
    } else {
        Err(RcError::BadState)
    }
}
