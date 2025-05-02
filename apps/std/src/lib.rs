#![no_std]

use alloc::{string::String, vec::Vec};
use common_std::ipc::{Channel, MessagePacket};
pub use common_std::*;
use embedded_io::{ErrorType, SliceWriteError};
use noline::{builder::EditorBuilder, history::UnboundedHistory, line_buffer::UnboundedBuffer, sync_editor::Editor};
use spin::{Lazy, Mutex};

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
        let mut data = alloc::vec![0];
        data.extend_from_slice(msg.as_bytes());
        stdio.write(&MessagePacket::new(data, Vec::new()))?;
        Ok(())
    } else {
        Err(RcError::BadState)
    }
}

pub fn read_line(line: &mut String) -> RcResult<()> {
    line.clear();
    
    static EDITOR: Lazy<Mutex<Editor<UnboundedBuffer, UnboundedHistory>>> = Lazy::new(|| {
        Mutex::new(EditorBuilder::new_unbounded()
            .with_unbounded_history()
            .build_sync(&mut ChannelIO{})
            .unwrap())
    });
    
    let mut io = ChannelIO {};

    loop {
        if let Ok(input) = EDITOR.lock().readline(">", &mut io) {
            line.push_str(input);
            break;
        }
    }

    Ok(())
}

pub unsafe fn stdio_channel() -> RcResult<Channel> {
    if let Some(ref stdio) = *STDIO.lock() {
        Ok(unsafe { Channel::from_handle(stdio.as_handle()) })
    } else {
        Err(RcError::BadState)
    }
}

pub struct ChannelIO {}

impl ErrorType for ChannelIO {
    type Error = SliceWriteError;
}

impl embedded_io::Read for ChannelIO {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if let Some(ref mut stdio) = *STDIO.lock() {
            let mut read = 0;

            while read == 0 {
                let mut data = alloc::vec![2];
                data.extend_from_slice(&(buf.len() - read).to_le_bytes());
                stdio
                    .write(&MessagePacket::new(data, alloc::vec![]))
                    .unwrap();

                let pack = loop {
                    if let Ok(pack) = stdio.read() {
                        break pack;
                    }
                };
                let data = pack.data;
                buf[read..read + data.len()].copy_from_slice(&data);
                read += data.len();
            }

            return Ok(read);
        }
        Err(SliceWriteError::Full)
    }
}

impl embedded_io::Write for ChannelIO {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        if let Some(ref mut stdio) = *STDIO.lock() {
            let mut data = alloc::vec![0];
            data.extend_from_slice(buf);
            stdio
                .write(&MessagePacket::new(data, alloc::vec![]))
                .unwrap();
            return Ok(buf.len());
        }
        Err(SliceWriteError::Full)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
