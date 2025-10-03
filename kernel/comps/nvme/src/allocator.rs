/*use alloc::vec::Vec;
use nvme::Allocator;
use ostd::{mm::{DmaDirection, DmaStream, FrameAllocOptions, HasDaddr, PAGE_SIZE}, sync::RwLock};

pub struct NvmeAllocator;

static DMA_STREAMS: RwLock<Vec<DmaStream>> = RwLock::new(Vec::new());

impl Allocator for NvmeAllocator {
    unsafe fn allocate(&self, size: usize) -> usize {
        let segments = FrameAllocOptions::new().alloc_segment((size + PAGE_SIZE - 1) / PAGE_SIZE).unwrap();

        let stream = DmaStream::map(segments, DmaDirection::Bidirectional, false).unwrap();

        let vaddr = stream.daddr();

        DMA_STREAMS.write().push(stream);
        0
    }
}*/
