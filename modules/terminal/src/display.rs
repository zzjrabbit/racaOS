use os_terminal::DrawTarget;
use ostd::{boot::boot_info, io::IoMem, mm::VmIo};

pub struct Display {
    width: usize,
    height: usize,
    buffer: IoMem,
}

impl Default for Display {
    fn default() -> Self {
        let frame_buffer = boot_info().framebuffer_arg.as_ref().unwrap();

        let address = frame_buffer.address;
        let len = frame_buffer.width * frame_buffer.height * frame_buffer.bpp / 8;

        log::info!(target: "kernel", "{}x{}x{}", frame_buffer.width, frame_buffer.height, frame_buffer.bpp);
        log::info!(target: "kernel", "Display MMIO address: {:x}..{:x}", address, address + len);

        Self {
            width: frame_buffer.width,
            height: frame_buffer.height,
            buffer: IoMem::acquire(address..address + len).unwrap(),
        }
    }
}

impl DrawTarget for Display {
    fn draw_pixel(&mut self, x: usize, y: usize, color: os_terminal::Rgb) {
        let (r, g, b) = color;

        let base = (x + y * self.width) * 4;
        self.buffer.write_val(base, &b).unwrap();
        self.buffer.write_val(base + 1, &g).unwrap();
        self.buffer.write_val(base + 2, &r).unwrap();
        self.buffer.write_val(base + 3, &0xFFu8).unwrap();
    }

    fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}
