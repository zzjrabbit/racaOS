use core::slice::from_raw_parts_mut;
use limine::request::FramebufferRequest;

#[used]
#[link_section = ".requests"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

#[derive(Debug, Clone, Copy)]
pub enum PixelFormat {
    Rgb,
    Bgr,
    U8,
    Unknown,
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Red,
    Yellow,
    Green,
    Blue,
    White,
}

impl Color {
    const fn get_color_rgb(&self) -> [u8; 3] {
        match self {
            Color::Red => [0xf4, 0x43, 0x36],
            Color::Yellow => [0xff, 0xc1, 0x07],
            Color::Green => [0x4c, 0xaf, 0x50],
            Color::Blue => [0x03, 0xa9, 0xf4],
            Color::White => [0xff, 0xff, 0xff],
        }
    }

    fn get_color_pixel(&self, pixel_format: PixelFormat, intensity: u8) -> [u8; 4] {
        let [r, g, b] = self
            .get_color_rgb()
            .map(|x| (x as u32 * intensity as u32 / 0xff) as u8);
        match pixel_format {
            PixelFormat::Rgb => [r, g, b, 0],
            PixelFormat::Bgr => [b, g, r, 0],
            PixelFormat::U8 => [intensity >> 4, 0, 0, 0],
            _ => panic!("Unknown pixel format: {:?}", pixel_format),
        }
    }
}

pub struct Display {
    pub buffer: &'static mut [u8],
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub bytes_per_pixel: usize,
    pub pixel_format: PixelFormat,
}

impl Display {
    pub fn get_main() -> Self {
        let response = FRAMEBUFFER_REQUEST.get_response().unwrap();
        let frame_buffer = response.framebuffers().last().take().unwrap();

        //let mode = frame_buffer.modes().unwrap().last().take().unwrap();
        //mode.

        let width = frame_buffer.width() as _;
        let height = frame_buffer.height() as _;

        let pixel_format = match (
            frame_buffer.red_mask_shift(),
            frame_buffer.green_mask_shift(),
            frame_buffer.blue_mask_shift(),
        ) {
            (0x00, 0x08, 0x10) => PixelFormat::Rgb,
            (0x10, 0x08, 0x00) => PixelFormat::Bgr,
            (0x00, 0x00, 0x00) => PixelFormat::U8,
            _ => PixelFormat::Unknown,
        };

        let pitch = frame_buffer.pitch() as usize;
        let bpp = frame_buffer.bpp() as usize;
        let stride = (pitch / 4) as _;
        let bytes_per_pixel = (bpp / 8) as _;

        let buffer_size = stride * height * bytes_per_pixel;
        let buffer = unsafe { from_raw_parts_mut(frame_buffer.addr(), buffer_size) };

        Self {
            buffer,
            width,
            height,
            stride,
            bytes_per_pixel,
            pixel_format,
        }
    }

    pub fn write_pixel(&mut self, x: usize, y: usize, color: Color, intensity: u8) {
        assert!(x < self.width);
        assert!(y < self.height);

        let byte_offset = (y * self.stride + x) * self.bytes_per_pixel;
        let write_range = byte_offset..(byte_offset + self.bytes_per_pixel);

        let color = color.get_color_pixel(self.pixel_format, intensity);
        self.buffer[write_range].copy_from_slice(&color[..self.bytes_per_pixel]);
        // ? Looking 6I'm going to eat you.[dodge]
    }
}
