use core::sync::atomic::{AtomicBool, Ordering};

use errors::Result;
use filesystem::Terminal;
use ostd::sync::WaitQueue;

use crate::{INPUT_BUFFER, TERMINAL_BUFFER};

pub struct OsTerminal;

static WAIT_QUEUE: WaitQueue = WaitQueue::new();
static DONE: AtomicBool = AtomicBool::new(false);

pub fn set_done() {
    DONE.store(true, Ordering::SeqCst);
    WAIT_QUEUE.wake_all();
}

impl Terminal for OsTerminal {
    fn read(&self, buffer: &mut [u8]) -> Result<usize> {
        DONE.store(false, Ordering::SeqCst);
        log::trace!("terminal read");
        WAIT_QUEUE.wait_until(|| DONE.load(Ordering::SeqCst).then_some(()));
        DONE.store(false, Ordering::SeqCst);
        log::trace!("terminal read");

        let mut read = 0;
        let mut input_buffer = INPUT_BUFFER.write();

        // TODO: Unblocked IO support.

        while read < buffer.len() && !input_buffer.is_empty() {
            buffer[read] = input_buffer.pop_front().unwrap();
            read += 1;
        }
        Ok(read)
    }

    fn write(&self, buffer: &[u8]) -> Result<usize> {
        TERMINAL_BUFFER.write().push_back(buffer.to_vec());
        Ok(buffer.len())
    }
}
