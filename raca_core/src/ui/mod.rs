use alloc::vec;

use crate::fs::operation::kernel_open;

use fontdue::Font;

pub fn init() {
    let font_inode = kernel_open("/raca/fonts/default.ttf".into()).unwrap();

    let font_size = font_inode.read().size();
    let mut font_buffer = vec![0; font_size];

    font_inode.read().read_at(0, &mut font_buffer);

    // Parse it into the font type.
    let fira_code = Font::from_bytes(font_buffer.as_slice(), fontdue::FontSettings::default()).unwrap();
    let (metrics, bitmap) = fira_code.rasterize('A', 16.0);
    for (idx,byte) in bitmap.iter().enumerate() {
        framework::print!("{}", if *byte > 200 { "A" } else { " " });
        if (idx + 1) % metrics.width == 0 {
            framework::println!();
        }
    }
}
