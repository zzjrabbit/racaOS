#![no_std]
#![no_main]

use common_std::{
    fb::FrameBuffer,
    ipc::{Channel, MessagePacket},
    signal::{Signal, wait_for_signal},
};
use spin::Mutex;

use alloc::{boxed::Box, vec::Vec};
use os_terminal::{DrawTarget, Terminal, font::TrueTypeFont};

extern crate alloc;

pub struct Buffer(FrameBuffer);

impl DrawTarget for Buffer {
    fn size(&self) -> (usize, usize) {
        (self.0.width(), self.0.height())
    }

    fn draw_pixel(&mut self, x: usize, y: usize, color: os_terminal::Rgb) {
        let pos = self.0.width * y + x;
        self.0.buffer[pos * 4 + 0] = color.2;
        self.0.buffer[pos * 4 + 1] = color.1;
        self.0.buffer[pos * 4 + 2] = color.0;
    }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn _start() {
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

    static BUFFER: Mutex<Vec<u8>> = Mutex::new(Vec::new());

    let mut terminal = Terminal::new(Buffer(FrameBuffer::get().unwrap()));

    terminal.set_font_manager(Box::new(TrueTypeFont::new(
        14.0,
        include_bytes!("FiraCodeNotoSans.ttf"),
    )));
    terminal.set_auto_flush(true);
    terminal.set_crnl_mapping(true);
    terminal.set_color_scheme(6);

    terminal.set_pty_writer(Box::new(|msg| {
        BUFFER.lock().extend_from_slice(msg.as_str().as_bytes());
    }));

    let channel = unsafe { Channel::from_handle(handles[0]) };

    let mut channels = alloc::vec![channel];

    loop {
        let handle = wait_for_signal(
            channels
                .iter()
                .map(|channel| channel.as_handle())
                .collect::<Vec<u32>>()
                .as_slice(),
            Signal::READABLE,
        )
        .unwrap();

        let channel = unsafe { Channel::from_handle(handle) };
        let msg = channel.read().unwrap();

        for new_handle in &msg.handles {
            unsafe {
                channels.push(Channel::from_handle(*new_handle));
            }
        }

        if msg.data.is_empty() {
            continue;
        }

        match msg.data[0] {
            0 => {
                terminal.process(&msg.data[1..]);
            }
            1 => {
                for scancode in msg.data[1..].iter() {
                    terminal.handle_keyboard(*scancode);
                }
            }
            2 => {
                let data = &msg.data[1..];
                let len = usize::from_le_bytes([
                    data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
                ]);

                let mut data = Vec::new();
                while !BUFFER.lock().is_empty() && data.len() < len {
                    data.push(BUFFER.lock().remove(0));
                }

                channel
                    .write(&MessagePacket::new(data, Vec::new()))
                    .unwrap();
            }
            _ => {}
        }
    }
}

#[panic_handler]
pub fn panic(_info: &core::panic::PanicInfo) -> ! {
    common_std::debug::debug("panic").unwrap();
    loop {}
}
