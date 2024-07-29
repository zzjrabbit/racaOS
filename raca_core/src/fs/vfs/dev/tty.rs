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
use x86_64::VirtAddr;

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

pub fn tty_thread() {
    let display = framework::drivers::display::Display::new();
    let frame_buffer = display.get_frame_buffer();
    let frame_buffer_address = VirtAddr::new(frame_buffer.as_ptr() as u64);
    let frame_buffer_size = frame_buffer_addr.len();

    ;
}
