#![no_std]

use component::{ComponentInitError, init_component};
use core::fmt::{self, Arguments, Write};
use filesystem::{Path, init_terminal, open_file};
use spin::{Lazy, Once};

use alloc::{boxed::Box, collections::vec_deque::VecDeque, string::String, sync::Arc, vec::Vec};
use os_terminal::{DrawTarget, Terminal, font::TrueTypeFont};
use ostd::{
    sync::{RwLock, WaitQueue},
    task::{Task, TaskOptions},
};

use crate::{
    display::Display,
    keyboard::SCANCODE_QUEUE,
    terminal::{OsTerminal, on_newline, on_read},
};

extern crate alloc;

mod display;
mod keyboard;
mod terminal;

#[init_component(kthread)]
pub fn terminal_init() -> Result<(), ComponentInitError> {
    Lazy::force(&TERMINAL_THREAD);
    init_terminal(Arc::new(OsTerminal));
    keyboard::init();

    Ok(())
}

fn wake_up() {
    WAIT_QUEUE.wake_all();
}

static TERMINAL_BUFFER: RwLock<VecDeque<Vec<u8>>> = RwLock::new(VecDeque::new());
static INPUT_BUFFER: RwLock<VecDeque<u8>> = RwLock::new(VecDeque::new());
static WAIT_QUEUE: WaitQueue = WaitQueue::new();

static SIZE_IN_CHARS: Once<(usize, usize)> = Once::new();
static SIZE_IN_PIXELS: Once<(usize, usize)> = Once::new();

pub struct TerminalWriter;

impl Write for TerminalWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        TERMINAL_BUFFER.write().push_back(s.as_bytes().to_vec());
        Ok(())
    }
}

fn terminal_flush(terminal: &mut Terminal<Display>) {
    while let Some(s) = TERMINAL_BUFFER.write().pop_front() {
        terminal.process(&s);
    }
}

fn terminal_event(terminal: &mut Terminal<Display>) {
    while let Some(scancode) = SCANCODE_QUEUE.pop() {
        terminal.handle_keyboard(scancode);
    }
}

fn logger(args: Arguments) {
    let msg = alloc::format!("{}", args);
    ostd::early_println!("{}", msg);
}

fn terminal_thread() {
    let data = {
        let font_file = open_file(&Path::from("/part0/SourceCodePro.otf")).unwrap();
        let mut data = alloc::vec![0u8; font_file.len() as usize];
        font_file.read_at(0, &mut data).unwrap();
        Box::leak(Box::new(data))
    };

    let display = Display::default();

    SIZE_IN_PIXELS.call_once(|| display.size());

    let mut terminal = Terminal::new(display);
    terminal.set_auto_flush(false);
    terminal.set_crnl_mapping(true);
    terminal.set_scroll_speed(5);
    terminal.set_font_manager(Box::new(TrueTypeFont::new(12.0, data)));
    terminal.set_logger(logger);

    SIZE_IN_CHARS.call_once(|| (terminal.rows(), terminal.columns()));

    terminal.set_pty_writer(Box::new(|s: String| {
        let mut new_line = false;
        for byte in s.bytes() {
            INPUT_BUFFER.write().push_back(byte);
            if byte == b'\n' {
                new_line = true;
            }
        }
        if new_line {
            on_newline();
        }
        on_read();
    }));

    loop {
        terminal.flush();
        terminal_event(&mut terminal);
        terminal_flush(&mut terminal);
        WAIT_QUEUE.wait_until(|| Some(()));
    }
}

static TERMINAL_THREAD: Lazy<Arc<Task>> = Lazy::new(|| {
    let thread = Arc::new(TaskOptions::new(terminal_thread).build().unwrap());
    thread.run();
    thread
});
