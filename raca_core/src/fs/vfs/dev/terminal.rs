use crate::fs::vfs::inode::Inode;
use crate::user::{get_current_thread, sleep};
use alloc::string::String;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use crossbeam_queue::ArrayQueue;
use framework::drivers::keyboard::get_scancode;
use framework::task::thread::ThreadState;
use framework::task::Thread;
use pc_keyboard::{layouts, DecodedKey, HandleControl, KeyCode, Keyboard, ScancodeSet1};
use spin::{Lazy, Mutex, RwLock};

static BYTES: Lazy<ArrayQueue<char>> = Lazy::new(|| ArrayQueue::new(1024));
static WAIT_LIST: Mutex<Vec<Weak<RwLock<Thread>>>> = Mutex::new(Vec::new());

pub fn keyboard_parse_thread() {
    fn push_char(ch: char) {
        BYTES.push(ch).expect("Buffer full");
        for thread in WAIT_LIST.lock().iter() {
            thread.upgrade().unwrap().write().state = ThreadState::Ready;
        }
        WAIT_LIST.lock().clear();
    }

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
                        DecodedKey::RawKey(raw_key) => match raw_key {
                            KeyCode::Backspace => push_char(8 as char),
                            KeyCode::Oem7 => push_char('\\'),
                            _ => {}
                        },
                        DecodedKey::Unicode(ch) => push_char(ch),
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
        &mut self,
        path: alloc::string::String,
        _father: Option<crate::fs::vfs::inode::InodeRef>,
    ) {
        self.path.clear();
        self.path.push_str(path.as_str());
    }

    fn when_umounted(&mut self) {
        self.path.clear();
    }

    fn get_path(&self) -> String {
        self.path.clone()
    }

    fn read_at(&self, _offset: usize, buf: &mut [u8]) {
        let mut write = 0;
        while write < buf.len() {
            if let Some(byte) = BYTES.pop() {
                buf[write] = byte as u8;
                write += 1;
            } else {
                let thread = Arc::downgrade(&get_current_thread());
                WAIT_LIST.lock().push(thread);
                sleep();
            }
        }
    }

    fn write_at(&self, _offset: usize, buf: &[u8]) {
        if let Ok(s) = core::str::from_utf8(buf) {
            framework::print!("{}", s);
        }
    }
}
