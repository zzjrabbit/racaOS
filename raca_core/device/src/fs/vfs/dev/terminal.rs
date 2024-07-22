use crate::fs::vfs::inode::Inode;
use alloc::string::String;
use framework::drivers::keyboard::{get_scancode, has_scancode};
use framework::ref_to_mut;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::Lazy;
use crossbeam_queue::ArrayQueue;

//static BYTES: Lazy<ArrayQueue<u8>> = Lazy::new(|| ArrayQueue::new(SCANCODE_QUEUE_SIZE));

pub struct Terminal {
    path: String,
    keyboard: Keyboard<layouts::Us104Key, ScancodeSet1>,
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            keyboard: Keyboard::new(
                ScancodeSet1::new(),
                layouts::Us104Key,
                HandleControl::Ignore,
            ),
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
            while !has_scancode() {}
            if let Some(scan_code) = get_scancode() {
                if let Ok(Some(key_event)) = ref_to_mut(self).keyboard.add_byte(scan_code) {
                    if let Some(key) = ref_to_mut(self).keyboard.process_keyevent(key_event) {
                        match key {
                            DecodedKey::RawKey(_) => {}
                            DecodedKey::Unicode(ch) => {
                                buf[write] = ch as u8;
                                write += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    fn write_at(&self, _offset: usize, buf: &[u8]) {
        if let Ok(s) = core::str::from_utf8(buf) {
            framework::print!("{}", s);
        }
    }
}
