use core::mem::swap;
use noto_sans_mono_bitmap::{get_raster, get_raster_width};
use noto_sans_mono_bitmap::{FontWeight, RasterHeight};

use super::cell::{Cell, Flags};

const FONT_WEIGHT: FontWeight = FontWeight::Bold;
const FONT_WIDTH: usize = get_raster_width(FONT_WEIGHT, FONT_HEIGHT);
const FONT_HEIGHT: RasterHeight = RasterHeight::Size20;

pub trait DrawTarget {
    fn size(&self) -> (usize, usize);
    fn draw_pixel(&mut self, x: usize, y: usize, color: (u8, u8, u8));
}

pub struct TextOnGraphic<D: DrawTarget> {
    width: usize,
    height: usize,
    graphic: D,
}

impl<D: DrawTarget> TextOnGraphic<D> {
    pub fn new(graphic: D, width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            graphic,
        }
    }

    pub fn width(&self) -> usize {
        self.width as usize / FONT_WIDTH
    }

    pub fn height(&self) -> usize {
        self.height as usize / FONT_HEIGHT as usize
    }

    pub fn clear(&mut self, cell: Cell) {
        for i in 0..self.height() {
            for j in 0..self.width() {
                self.write(i, j, cell);
            }
        }
    }

    pub fn write(&mut self, row: usize, col: usize, cell: Cell) {
        if row >= self.height() || col >= self.width() {
            return;
        }

        let mut foreground = cell.foreground.to_rgb();
        let mut background = cell.background.to_rgb();

        if cell.flags.contains(Flags::INVERSE) {
            swap(&mut foreground, &mut background);
        }

        if cell.flags.contains(Flags::HIDDEN) {
            foreground = background;
        }

        let char_raster = get_raster(cell.content, FONT_WEIGHT, FONT_HEIGHT).unwrap();
        let (x_start, y_start) = (col * FONT_WIDTH, row * FONT_HEIGHT as usize);

        for (y, lines) in char_raster.raster().iter().enumerate() {
            for (x, intensity) in lines.iter().enumerate() {
                let calculate_color = |fg, bg| {
                    let weight = (fg as i32 - bg as i32) * *intensity as i32 / 0xff;
                    ((bg as i32 + weight).clamp(0, 255)) as u8
                };

                let r = calculate_color(foreground.0, background.0);
                let g = calculate_color(foreground.1, background.1);
                let b = calculate_color(foreground.2, background.2);

                self.graphic.draw_pixel(x_start + x, y_start + y, (r, g, b));
            }
        }
    }
}
