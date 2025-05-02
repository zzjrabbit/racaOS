use core::slice::from_raw_parts_mut;

use crate::{RcResult, syscall};

pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: &'static mut [u8],
    handle: u32,
}

impl FrameBuffer {
    pub fn get() -> RcResult<Self> {
        let mut fb = 0u32;
        syscall!(13, &raw mut fb)?;
        let ptr = syscall!(12, fb)?;
        let width = syscall!(10, fb)?;
        let height = syscall!(11, fb)?;

        let buffer = unsafe { from_raw_parts_mut(ptr as *mut u8, width * height * 4) };

        Ok(FrameBuffer {
            width,
            height,
            buffer,
            handle: fb,
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}
