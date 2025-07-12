#![no_std]

use alloc::{string::String, vec::Vec};
pub use common_std::*;
use common_std::{
    ipc::{Channel, MessagePacket},
    signal::{Signal, wait_for_signal},
};
use readline::Readline;
use spin::{Lazy, Mutex};

extern crate alloc;

pub mod print;
mod readline;

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
        let mut data = alloc::vec![0];
        data.extend_from_slice(msg.as_bytes());
        stdio.write(&MessagePacket::new(data, Vec::new()))?;
        Ok(())
    } else {
        Err(RcError::BadState)
    }
}

pub fn read_line(line: &mut String) -> RcResult<()> {
    static READLINE: Lazy<Mutex<Readline>> = Lazy::new(|| Mutex::new(Readline::new()));

    READLINE.lock().read_line(line);
    Ok(())
}

pub unsafe fn stdio_channel() -> RcResult<Channel> {
    if let Some(ref stdio) = *STDIO.lock() {
        Ok(unsafe { Channel::from_handle(stdio.as_handle()) })
    } else {
        Err(RcError::BadState)
    }
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("user panic: {}", info);
    loop {}
}
