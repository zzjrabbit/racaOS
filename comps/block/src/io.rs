use alloc::boxed::Box;
use ostd::mm::{DmaDirection, DmaStream, FrameAllocOptions, VmIo};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BlockOperation {
    Read,
    Write,
}

pub struct BlockIo {
    inner: BlockIoInner,
}

struct BlockIoInner {
    operation: BlockOperation,
    dma_stream: DmaStream,
    on_complete: Option<Box<dyn Fn()>>,
}

impl BlockIo {
    pub fn new(operation: BlockOperation, block_num: usize) -> Self {
        let frames = FrameAllocOptions::new().alloc_segment(block_num).unwrap();
        let dma_stream = DmaStream::map(frames.into(), DmaDirection::Bidirectional, false).unwrap();

        Self {
            inner: BlockIoInner {
                operation,
                dma_stream,
                on_complete: None,
            },
        }
    }

    pub fn on_complete(&mut self, callback: Box<dyn Fn()>) {
        self.inner.on_complete = Some(callback);
    }

    pub fn read(&self, buf: &mut [u8]) {
        self.dma_stream().read_bytes(0, buf).unwrap();
    }

    pub fn write(&self, buf: &[u8]) {
        self.dma_stream().write_bytes(0, buf).unwrap();
    }
}

impl BlockIo {
    pub fn complete(&self) {
        if let Some(callback) = self.inner.on_complete.as_ref() {
            callback();
        }
    }

    pub fn dma_stream(&self) -> &DmaStream {
        &self.inner.dma_stream
    }

    pub fn operation(&self) -> BlockOperation {
        self.inner.operation
    }
}
