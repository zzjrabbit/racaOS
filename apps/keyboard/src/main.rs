#![no_std]
#![no_main]

use std::{
    hw::{IoPort, Irq},
    ipc::MessagePacket,
    signal::{Signal, clear_signal, wait_for_signal},
};

use alloc::vec::Vec;

extern crate alloc;

#[unsafe(no_mangle)]
pub fn main(_handles: Vec<u32>) {
    //std::print("Hello World From Keyboard!\n").unwrap();
    let irq = Irq::new(1).unwrap();
    let keyboard_port = IoPort::new(0x60).unwrap();

    loop {
        wait_for_signal(&[irq.as_handle()], Signal::INTERRUPT_PRESENT).unwrap();
        clear_signal(irq.as_handle(), Signal::INTERRUPT_PRESENT).unwrap();

        let scancode = keyboard_port.read::<u8>().unwrap();
        let data = alloc::vec![1, scancode];
        unsafe {
            std::stdio_channel()
                .unwrap()
                .write(&MessagePacket::new(data, Vec::new()))
                .unwrap()
        };
    }
}
