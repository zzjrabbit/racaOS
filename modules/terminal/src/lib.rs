#![no_std]

use component::{ComponentInitError, init_component};
use core::{
    fmt::{self, Arguments, Write},
    sync::atomic::{AtomicBool, Ordering},
};
use filesystem::{Path, init_terminal, open_file};
use spin::Lazy;

use alloc::{boxed::Box, collections::vec_deque::VecDeque, string::String, sync::Arc, vec::Vec};
use os_terminal::{Terminal, font::TrueTypeFont};
use ostd::{
    sync::RwLock,
    task::{Task, TaskOptions},
};

use crate::{
    display::Display,
    keyboard::SCANCODE_QUEUE,
    terminal::{OsTerminal, set_done},
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

static TERMINAL_BUFFER: RwLock<VecDeque<Vec<u8>>> = RwLock::new(VecDeque::new());
static INPUT_BUFFER: RwLock<VecDeque<u8>> = RwLock::new(VecDeque::new());
static NEED_FLUSH: AtomicBool = AtomicBool::new(false);

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
        NEED_FLUSH.store(true, Ordering::Relaxed);
    }

    if NEED_FLUSH.swap(false, Ordering::Relaxed) {
        terminal.flush();
    }
}

fn terminal_event(terminal: &mut Terminal<Display>) {
    while let Some(scancode) = SCANCODE_QUEUE.pop() {
        terminal.handle_keyboard(scancode);
        NEED_FLUSH.store(true, Ordering::Relaxed);
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

    let mut terminal = Terminal::new(Display::default());
    terminal.set_auto_flush(false);
    terminal.set_crnl_mapping(true);
    terminal.set_scroll_speed(5);
    terminal.set_font_manager(Box::new(TrueTypeFont::new(12.0, data)));
    terminal.set_logger(logger);

    terminal.set_pty_writer(Box::new(|s: String| {
        if s.contains("\n") {
            set_done();
        }
        for byte in s.bytes() {
            INPUT_BUFFER.write().push_back(byte);
        }
        TerminalWriter.write_str(&s).unwrap()
    }));

    loop {
        terminal_event(&mut terminal);
        terminal_flush(&mut terminal);
        Task::yield_now();
    }
}

static TERMINAL_THREAD: Lazy<Arc<Task>> = Lazy::new(|| {
    let thread = Arc::new(TaskOptions::new(terminal_thread).build().unwrap());
    thread.run();
    thread
});
