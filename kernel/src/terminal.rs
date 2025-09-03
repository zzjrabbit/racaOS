use core::{
    fmt::{self, Write},
    sync::atomic::{AtomicBool, Ordering},
};

use alloc::{boxed::Box, collections::vec_deque::VecDeque, string::String, sync::Arc, vec::Vec};
use os_terminal::{DrawTarget, Terminal, font::BitmapFont};
use spin::{Lazy, RwLock};
use zodiac::task::{Task, TaskBuilder};

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

#[derive(Default)]
struct Display(zodiac::framebuffer::FrameBuffer);

impl DrawTarget for Display {
    fn draw_pixel(&mut self, x: usize, y: usize, color: os_terminal::Rgb) {
        self.0.draw_pixel(x, y, color);
    }

    fn size(&self) -> (usize, usize) {
        self.0.size()
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

fn terminal_thread() -> ! {
    /*let mut display = Display::default();

    let (width, height) = display.size();

    for y in 0..height {
        for x in 0..width {
            display.draw_pixel(x, y, (0x70, 0xb0, 0xd0));
        }
    }*/
    let mut terminal = Terminal::new(Display::default());
    terminal.set_auto_flush(false);
    terminal.set_crnl_mapping(true);
    terminal.set_scroll_speed(5);
    terminal.set_font_manager(Box::new(BitmapFont));
    terminal.set_color_scheme(6);

    terminal.set_pty_writer(Box::new(|s: String| TerminalWriter.write_str(&s).unwrap()));

    loop {
        terminal_flush(&mut terminal);
        Task::current().r#yield();
    }
}

static TERMINAL_THREAD: Lazy<Arc<Task>> = Lazy::new(|| {
    let thread = TaskBuilder::default()
        .entry(terminal_thread)
        .kernel_mode()
        .kernel_stack_size(256 * 1024)
        .build()
        .unwrap();
    thread.spawn();
    thread
});

pub fn init() {
    Lazy::force(&TERMINAL_THREAD);
}
