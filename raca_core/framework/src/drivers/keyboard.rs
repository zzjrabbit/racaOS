use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;

const SCANCODE_QUEUE_SIZE: usize = 128;

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

pub fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            crate::println!("Scancode queue full, dropping keyboard input!");
        }
    } else {
        crate::println!("Scancode queue not initialized!");
    }
}

pub fn init() {
    SCANCODE_QUEUE.init_once(|| ArrayQueue::new(SCANCODE_QUEUE_SIZE));
}
