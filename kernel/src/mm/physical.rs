use core::range::Range;

use alloc::sync::Arc;

use crate::error::{RcError, RcResult};

crate::kernel_object! {
    pub struct PhysicalMemory {
        start_address: usize = start_address,
        frame_count: usize = frame_count,
    }

    fn new(start_address: usize, frame_count: usize) {}
}

impl PhysicalMemory {
    pub fn allocate(count: usize) -> RcResult<Arc<Self>> {
        let start_address = crate::hal::alloc_frames(count).ok_or(RcError::AllocationFailed)?;
        Ok(Self::new(start_address, count))
    }

    pub fn create_child(&self, frame_range: Range<usize>) -> Arc<Self> {
        let start_frame = frame_range.start;
        let end_frame = frame_range.end;

        let child_start_address = self.start_address + start_frame + 4096 * start_frame;
        let frame_count = end_frame - start_frame;

        Self::new(child_start_address, frame_count)
    }

    pub fn len(&self) -> usize {
        self.frame_count * 4096
    }

    pub fn is_empty(&self) -> bool {
        self.frame_count == 0
    }

    pub fn start_address(&self) -> usize {
        self.start_address
    }

    pub fn frame_count(&self) -> usize {
        self.frame_count
    }
}
