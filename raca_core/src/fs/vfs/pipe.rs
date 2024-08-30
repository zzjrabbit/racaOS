use alloc::{collections::vec_deque::VecDeque, string::String, sync::Weak, vec::Vec};
use framework::{
    ref_to_mut,
    task::{thread::ThreadState, Thread},
};
use spin::RwLock;

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
    fn when_mounted(
        &mut self,
        path: alloc::string::String,
        _father: Option<super::inode::InodeRef>,
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

    fn size(&self) -> usize {
        self.buffer.len()
    }

    fn read_at(&self, _offset: usize, buf: &mut [u8]) -> usize {
        let mut write = 0;
        while let Some(data) = ref_to_mut(self).buffer.pop_back() {
            buf[write] = data;
            write += 1;
        }
        write
    }

    fn write_at(&self, _offset: usize, buf: &[u8]) -> usize {
        for &data in buf.iter() {
            ref_to_mut(self).buffer.push_front(data);
        }
        for thread in self.reader_requier.iter() {
            thread.upgrade().unwrap().write().state = ThreadState::Ready;
        }
        ref_to_mut(self).reader_requier.clear();
        buf.len()
    }
}
