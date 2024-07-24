use crossbeam_queue::ArrayQueue;
use spin::Lazy;

const SCANCODE_QUEUE_SIZE: usize = 128;

static SCANCODE_QUEUE: Lazy<ArrayQueue<u8>> = Lazy::new(|| ArrayQueue::new(SCANCODE_QUEUE_SIZE));

pub fn add_scancode(scancode: u8) {
    if let Err(_) = SCANCODE_QUEUE.push(scancode) {
        crate::println!("Scancode queue full, dropping keyboard input!");
    }
}

pub fn get_scancode() -> Option<u8> {
    SCANCODE_QUEUE.pop()
}

pub fn has_scancode() -> bool {
    !SCANCODE_QUEUE.is_empty()
}
