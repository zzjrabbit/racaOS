use alloc::{
    collections::vec_deque::VecDeque,
    string::String,
    sync::{Arc, Weak},
    vec::Vec,
};
use framework::{
    ref_to_mut,
    task::{thread::ThreadState, Thread},
};
use spin::RwLock;

use crate::user::{get_current_thread, sleep};

use super::inode::Inode;

pub struct Pipe {
    buffer: VecDeque<u8>,
    path: String,
    reader_requier: Vec<Weak<RwLock<Thread>>>,
}

impl Pipe {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
            path: String::new(),
            reader_requier: Vec::new(),
        }
    }
}

impl Inode for Pipe {
    fn when_mounted(&self, path: alloc::string::String, _father: Option<super::inode::InodeRef>) {
        ref_to_mut(self).path = path;
    }
    fn when_umounted(&self) {
        ref_to_mut(self).path.clear();
    }

    fn get_path(&self) -> String {
        self.path.clone()
    }

    fn size(&self) -> usize {
        self.buffer.len()
    }

    fn read_at(&self, _offset: usize, buf: &mut [u8]) {
        let mut write = 0;
        while write < buf.len() {
            if let Some(data) = ref_to_mut(self).buffer.pop_back() {
                buf[write] = data;
                write += 1;
            } else {
                let thread = Arc::downgrade(&get_current_thread());
                ref_to_mut(self).reader_requier.push(thread);
                sleep();
            }
        }
    }

    fn write_at(&self, _offset: usize, buf: &[u8]) {
        log::info!("WriteOK");
        for &data in buf.iter() {
            ref_to_mut(self).buffer.push_front(data);
        }
        for thread in self.reader_requier.iter() {
            thread.upgrade().unwrap().write().state = ThreadState::Ready;
            log::info!("Get up, you damn lazy shell!");
        }
        ref_to_mut(self).reader_requier.clear();
        log::info!("WriteOK");
    }
}
