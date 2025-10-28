use core::hint::spin_loop;

use alloc::{sync::Arc, vec::Vec};
use errors::Result;
use filesystem::{CInputFlags, CLocalFlags, COutputFlags, CTermios, IoEvent, Terminal};
use ostd::sync::{RwArc, RwLock, Waiter, Waker};

use crate::{INPUT_BUFFER, SIZE_IN_CHARS, SIZE_IN_PIXELS, TERMINAL_BUFFER, wake_up};

pub struct OsTerminal;

static READ_WAKERS: RwLock<Vec<Arc<Waker>>> = RwLock::new(Vec::new());

pub fn set_done() {
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
        CTermios {
            c_iflags: CInputFlags::ICRNL | CInputFlags::IXON | CInputFlags::IUTF8,
            c_oflags: COutputFlags::OPOST | COutputFlags::ONLCR,
            c_lflags: CLocalFlags::ECHO
                | CLocalFlags::ECHOE
                | CLocalFlags::ECHOK
                | CLocalFlags::IEXTEN
                | CLocalFlags::ECHOKE
                | CLocalFlags::ICANON,
            ..Default::default()
        }
    }

    fn register_waker(&self, required: IoEvent, event: RwArc<IoEvent>, _waker: Arc<Waker>) {
        event.write().insert(required);
        return;
    }
}
