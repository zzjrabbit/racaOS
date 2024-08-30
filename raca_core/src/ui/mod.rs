use alloc::vec;
use framework::{console::set_font, drivers::display::Display};

use crate::fs::operation::kernel_open;

use fontdue::Font;
use framework::{print, println};

pub fn init() {
    let font_inode = kernel_open("/raca/fonts/default.ttf".into()).unwrap();

    let font_size = font_inode.read().size();
    let mut font_buffer = vec![0; font_size];

    font_inode.read().read_at(0, &mut font_buffer);

    // Parse it into the font type.
    let fira_code =
        Font::from_bytes(font_buffer.as_slice(), fontdue::FontSettings::default()).unwrap();

    Display::new().get_frame_buffer().fill(0);
    set_font(10.0, font_buffer.leak());

    let (metrics_r, bitmap_r) = fira_code.rasterize('R', 20.0);
    let (metrics_a, bitmap_a) = fira_code.rasterize('A', 20.0);
    let (metrics_c, bitmap_c) = fira_code.rasterize('C', 20.0);

    let lines_count = metrics_r.height;

    for y in 0..lines_count {
        for x in 0..metrics_r.width {
            if bitmap_r[y * metrics_r.width + x] > 160 {
                print!("R");
            } else {
                print!(" ");
            }
        }

        print!(" ");

        for x in 0..metrics_a.width {
            if bitmap_a[y * metrics_a.width + x] > 160 {
                print!("A");
            } else {
                print!(" ");
            }
        }

        print!(" ");

        for x in 0..metrics_c.width {
            if bitmap_c[y * metrics_c.width + x] > 160 {
                print!("C");
            } else {
                print!(" ");
            }
        }

        for x in 0..metrics_a.width {
            if bitmap_a[y * metrics_a.width + x] > 160 {
                print!("A");
            } else {
                print!(" ");
            }
        }

        println!();
    }
}
