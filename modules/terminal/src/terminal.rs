use core::hint::spin_loop;

use alloc::{sync::Arc, vec::Vec};
use errors::Result;
use filesystem::{CTermios, IoEvent, Pollee, Terminal};
use ostd::sync::{RwLock, Waiter, Waker};

use crate::{INPUT_BUFFER, SIZE_IN_CHARS, SIZE_IN_PIXELS, TERMINAL_BUFFER, TERMIOS, wake_up};

pub struct OsTerminal;

static READ_WAKERS: RwLock<Vec<Arc<Waker>>> = RwLock::new(Vec::new());
static POLLEE: Pollee = Pollee::new();

pub fn on_read() {
    POLLEE.update_event(|event| *event |= IoEvent::IN);
}

pub fn on_newline() {
    READ_WAKERS.write().drain(..).for_each(|waker| {
        waker.wake_up();
    });
}

impl Terminal for OsTerminal {
    fn read(&self, buffer: &mut [u8]) -> Result<usize> {
        if INPUT_BUFFER.read().is_empty() {
            let (waiter, waker) = Waiter::new_pair();
            READ_WAKERS.write().push(waker);
            waiter.wait();
        }

        let mut read = 0;
        let mut input_buffer = INPUT_BUFFER.write();

        // TODO: Unblocked IO support.

        while read < buffer.len() && !input_buffer.is_empty() {
            buffer[read] = input_buffer.pop_front().unwrap();
            read += 1;
        }

        if input_buffer.is_empty() {
            POLLEE.update_event(|event| event.remove(IoEvent::IN));
        }

        Ok(read)
    }

    fn write(&self, buffer: &[u8]) -> Result<usize> {
        TERMINAL_BUFFER.write().push_back(buffer.to_vec());
        wake_up();
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
        TERMIOS.read().clone()
    }

    fn set_termios(&self, termios: &CTermios) {
        *TERMIOS.write() = *termios;
    }

    fn register_poller(&self, poller: Arc<filesystem::Poller>) {
        POLLEE.register_poller(poller);
    }
}
