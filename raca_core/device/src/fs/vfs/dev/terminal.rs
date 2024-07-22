use crate::fs::vfs::inode::Inode;
use alloc::string::String;
use crossbeam_queue::ArrayQueue;
use framework::drivers::keyboard::{get_scancode, has_scancode};
use framework::ref_to_mut;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::Lazy;

static BYTES: Lazy<ArrayQueue<char>> = Lazy::new(|| ArrayQueue::new(1024));

pub fn keyboard_parse_thread() {
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    loop {
        if let Some(scan_code) = get_scancode() {
            if let Ok(Some(key_event)) = keyboard.add_byte(scan_code) {
                if let Some(key) = keyboard.process_keyevent(key_event) {
                    match key {
                        DecodedKey::RawKey(_) => {}
                        DecodedKey::Unicode(ch) => BYTES.push(ch).expect("Buffer full"),
                    }
                }
            }
        }
    }
}

pub struct Terminal {
    path: String,
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            path: String::new(),
        }
    }
}

impl Inode for Terminal {
    fn when_mounted(
        &self,
        path: alloc::string::String,
        _father: Option<crate::fs::vfs::inode::InodeRef>,
    ) {
        ref_to_mut(self).path = path;
    }

    fn when_umounted(&self) {
        ref_to_mut(self).path = String::new();
    }

    fn get_path(&self) -> String {
        self.path.clone()
    }

    fn read_at(&self, _offset: usize, buf: &mut [u8]) {
        let mut write = 0;
        while write < buf.len() {
            while !has_scancode() {
                for _ in 0..10000 {
                    x86_64::instructions::nop();
                }
            }
            if let Some(byte) = BYTES.pop() {
                buf[write] = byte as u8;
                write +=1;
            }
            for _ in 0..10000 {
                x86_64::instructions::nop();
            }
        }
    }

    fn write_at(&self, _offset: usize, buf: &[u8]) {
        if let Ok(s) = core::str::from_utf8(buf) {
            framework::print!("{}", s);
        }
    }
}
