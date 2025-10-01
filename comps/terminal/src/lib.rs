#![no_std]

use component::{ComponentInitError, init_component};
use core::{
    fmt::{self, Write},
    sync::atomic::{AtomicBool, Ordering},
};
use spin::Lazy;

use alloc::{boxed::Box, collections::vec_deque::VecDeque, string::String, sync::Arc, vec::Vec};
use os_terminal::{DrawTarget, Terminal, font::BitmapFont};
use ostd::{
    boot::boot_info,
    io::IoMem,
    mm::VmIo,
    sync::RwLock,
    task::{Task, TaskOptions},
};

extern crate alloc;

#[init_component(kthread)]
pub fn terminal_init() -> Result<(), ComponentInitError> {
    Lazy::force(&TERMINAL_THREAD);

    Ok(())
}

static TERMINAL_BUFFER: RwLock<VecDeque<Vec<u8>>> = RwLock::new(VecDeque::new());
static NEED_FLUSH: AtomicBool = AtomicBool::new(false);

pub fn terminal_write(str: Vec<u8>) {
    TERMINAL_BUFFER.write().push_back(str);
}

pub struct TerminalWriter;

impl Write for TerminalWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        TERMINAL_BUFFER.write().push_back(s.as_bytes().to_vec());
        Ok(())
    }
}

impl DrawTarget for Display {
    fn draw_pixel(&mut self, x: usize, y: usize, color: os_terminal::Rgb) {
        let (r, g, b) = color;

        let base = (x + y * self.width) * 4;
        self.buffer.write_val(base + 0, &b).unwrap();
        self.buffer.write_val(base + 1, &g).unwrap();
        self.buffer.write_val(base + 2, &r).unwrap();
        self.buffer.write_val(base + 3, &0xFFu8).unwrap();
    }

    fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}

fn terminal_flush(terminal: &mut Terminal<Display>) {
    while let Some(s) = TERMINAL_BUFFER.write().pop_back() {
        terminal.process(&s);
        NEED_FLUSH.store(true, Ordering::Relaxed);
    }

    if NEED_FLUSH.swap(false, Ordering::Relaxed) {
        terminal.flush();
    }
}

fn terminal_thread() {
    let mut terminal = Terminal::new(Display::default());
    terminal.set_auto_flush(false);
    terminal.set_crnl_mapping(true);
    terminal.set_scroll_speed(5);
    terminal.set_font_manager(Box::new(BitmapFont));
    terminal.set_color_scheme(6);

    terminal.set_pty_writer(Box::new(|s: String| TerminalWriter.write_str(&s).unwrap()));

    loop {
        terminal_flush(&mut terminal);
        Task::yield_now();
    }
}

static TERMINAL_THREAD: Lazy<Arc<Task>> = Lazy::new(|| {
    let thread = Arc::new(TaskOptions::new(terminal_thread).build().unwrap());
    thread.run();
    thread.into()
});

pub struct Display {
    width: usize,
    height: usize,
    buffer: IoMem,
}

impl Default for Display {
    fn default() -> Self {
        let frame_buffer = boot_info().framebuffer_arg.as_ref().unwrap();

        let address = frame_buffer.address;
        let len = frame_buffer.width * frame_buffer.height * frame_buffer.bpp;

        Self {
            width: frame_buffer.width,
            height: frame_buffer.height,
            buffer: IoMem::acquire(address..address + len).unwrap(),
        }
    }
}
