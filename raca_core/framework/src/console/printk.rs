use core::fmt::{self, Write};

use super::display::*;
use conquer_once::spin::OnceCell;
use noto_sans_mono_bitmap::{get_raster, get_raster_width, FontWeight, RasterHeight};
use spin::Mutex;
use x86_64::instructions::interrupts::without_interrupts;

const FONT_WEIGHT: FontWeight = FontWeight::Bold;
const FONT_WIDTH: usize = get_raster_width(FONT_WEIGHT, FONT_HEIGHT);
const FONT_HEIGHT: RasterHeight = RasterHeight::Size16;
pub const DEFAULT_COLOR: Color = Color::White;

static PRINTK: OnceCell<Mutex<Printk>> = OnceCell::uninit();

pub fn init() {
    let printk = Printk {
        row_position: 0,
        column_position: 0,
        color: DEFAULT_COLOR,
        display: Display::get_main(),
    };

    PRINTK.try_init_once(|| Mutex::new(printk)).unwrap();
}

pub struct Printk {
    row_position: usize,
    column_position: usize,
    color: Color,
    display: Display,
}

impl Printk {
    #[inline]
    fn clear_screen(&mut self) {
        self.display.buffer.fill(0);
        self.row_position = 0;
        self.column_position = 0;
    }

    #[inline]
    fn new_line(&mut self) {
        self.row_position = 0;
        self.column_position += FONT_HEIGHT as usize;
    }

    fn back_space(&mut self) {
        if self.row_position > 0 {
            self.row_position -= FONT_WIDTH;
        }
        for y in 0..FONT_HEIGHT as usize {
            for x in 0..FONT_WIDTH {
                self.display.write_pixel(
                    self.row_position + x,
                    self.column_position + y,
                    self.color,
                    0,
                );
            }
        }
    }

    fn write_byte(&mut self, byte: char) {
        if self.row_position >= self.display.width - FONT_WIDTH {
            self.new_line();
        }
        if self.column_position >= self.display.height {
            self.clear_screen();
        }
        let rendered = get_raster(byte, FONT_WEIGHT, FONT_HEIGHT).unwrap();
        for (y, lines) in rendered.raster().iter().enumerate() {
            for (x, column) in lines.iter().enumerate() {
                self.display.write_pixel(
                    self.row_position + x,
                    self.column_position + y,
                    self.color,
                    *column,
                );
            }
        }
        self.row_position += rendered.width();
    }
}

impl fmt::Write for Printk {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        for byte in string.chars() {
            match byte {
                '\n' => self.new_line(),
                '\x08' => self.back_space(),
                _ => self.write_byte(byte),
            }
        }
        Ok(())
    }
}

#[inline]
pub fn _print(color: Color, args: fmt::Arguments) {
    without_interrupts(|| {
        let mut printk = PRINTK.try_get().unwrap().lock();
        printk.color = color;
        printk.write_fmt(args).unwrap();
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        $crate::console::printk::_print(
            $crate::console::printk::DEFAULT_COLOR,
            format_args!($($arg)*)
        )
    )
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)))
}
