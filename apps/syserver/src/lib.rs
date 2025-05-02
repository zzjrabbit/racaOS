#![no_std]

use alloc::vec::Vec;
use common_std::ipc::{Channel, MessagePacket};
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
    let handle_num;

    loop {
        if let Ok(msg) = channel_with_father.read() {
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
            handle_num = usize::from_le_bytes(data);
            break;
        }
    }

    while handles.len() < handle_num {
        if let Ok(msg) = channel_with_father.read() {
            for handle in msg.handles {
                handles.push(handle);
            }
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
