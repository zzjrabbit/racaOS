#![no_std]
#![no_main]

use std::{
    hw::{IoPort, Irq},
    ipc::MessagePacket,
};

use alloc::vec::Vec;

extern crate alloc;

#[unsafe(no_mangle)]
pub fn main(_handles: Vec<u32>) {
    //std::print("Hello World From Keyboard!\n").unwrap();
    let irq = Irq::new(1).unwrap();
    let port = IoPort::new(0x60).unwrap();

    loop {
        irq.wait().unwrap();

        let scancode = port.read::<u8>().unwrap();
        let data = alloc::vec![1, scancode];
        unsafe {
            std::stdio_channel()
                .unwrap()
                .write(&MessagePacket::new(data, Vec::new()))
                .unwrap()
        };
    }
}
