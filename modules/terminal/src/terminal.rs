use core::{
    hint::spin_loop,
    sync::atomic::{AtomicBool, Ordering},
};

use alloc::sync::Arc;
use errors::Result;
use filesystem::{CInputFlags, CLocalFlags, COutputFlags, CTermios, IoEvent, Terminal};
use ostd::sync::{RwArc, WaitQueue, Waker};

use crate::{INPUT_BUFFER, SIZE_IN_CHARS, SIZE_IN_PIXELS, TERMINAL_BUFFER};

pub struct OsTerminal;

static WAIT_QUEUE: WaitQueue = WaitQueue::new();
static DONE: AtomicBool = AtomicBool::new(false);

pub fn set_done() {
    DONE.store(true, Ordering::SeqCst);
    WAIT_QUEUE.wake_all();
}

impl Terminal for OsTerminal {
    fn read(&self, buffer: &mut [u8]) -> Result<usize> {
        log::debug!("reading terminal.");

        if INPUT_BUFFER.read().is_empty() {
            DONE.store(false, Ordering::SeqCst);
            WAIT_QUEUE.wait_until(|| DONE.load(Ordering::SeqCst).then_some(()));
            DONE.store(false, Ordering::SeqCst);
        }

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

    fn size_in_chars(&self) -> (usize, usize) {
        while let None = SIZE_IN_CHARS.get() {
            spin_loop();
        }
        *SIZE_IN_CHARS.get().unwrap()
    }

    fn size_in_pixels(&self) -> (usize, usize) {
        while let None = SIZE_IN_PIXELS.get() {
            spin_loop();
        }
        *SIZE_IN_PIXELS.get().unwrap()
    }

    fn termios(&self) -> CTermios {
        CTermios {
            c_iflags: CInputFlags::ICRNL | CInputFlags::IXON | CInputFlags::IUTF8,
            c_oflags: COutputFlags::OPOST | COutputFlags::ONLCR,
            c_lflags: CLocalFlags::ECHOE
                | CLocalFlags::ECHOK
                | CLocalFlags::IEXTEN
                | CLocalFlags::ECHOKE
                | CLocalFlags::ICANON,
            ..Default::default()
        }
    }

    fn register_waker(&self, required: IoEvent, event: RwArc<IoEvent>, waker: Arc<Waker>) {
        event.write().insert(required);
        waker.wake_up();
        return;
    }
}
