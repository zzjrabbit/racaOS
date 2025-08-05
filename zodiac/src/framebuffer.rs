use limine::request::FramebufferRequest;

#[used]
#[unsafe(link_section = ".requests")]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

type Rgb = (u8, u8, u8);

pub struct FrameBuffer {
    width: usize,
    height: usize,
    stride: usize,
    buffer: *mut u32,
    shifts: (u8, u8, u8),
    convert_color: fn(Rgb, Rgb) -> u32,
}

impl FrameBuffer {
    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn draw_pixel(&mut self, x: usize, y: usize, color: (u8, u8, u8)) {
        let color = (self.convert_color)(self.shifts, color);
        unsafe { self.buffer.add(y * self.stride + x).write(color) }
    }
}

impl Default for FrameBuffer {
    fn default() -> Self {
        let response = FRAMEBUFFER_REQUEST.get_response().unwrap();
        let frame_buffer = response.framebuffers().next().unwrap();

        let red_mask_size = frame_buffer.red_mask_size();
        let green_mask_size = frame_buffer.green_mask_size();
        let blue_mask_size = frame_buffer.blue_mask_size();

        let shifts = (
            frame_buffer.red_mask_shift() + (red_mask_size - 8),
            frame_buffer.green_mask_shift() + (green_mask_size - 8),
            frame_buffer.blue_mask_shift() + (blue_mask_size - 8),
        );

        let convert_color = |shifts: (u8, u8, u8), color: (u8, u8, u8)| {
            ((color.0 as u32) << shifts.0)
                | ((color.1 as u32) << shifts.1)
                | ((color.2 as u32) << shifts.2)
        };

        Self {
            shifts,
            convert_color,
            width: frame_buffer.width() as usize,
            height: frame_buffer.height() as usize,
            buffer: frame_buffer.addr() as *mut u32,
            stride: frame_buffer.pitch() as usize / size_of::<u32>(),
        }
    }
}
