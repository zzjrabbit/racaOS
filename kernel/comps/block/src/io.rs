use alloc::{boxed::Box, vec::Vec};
use ostd::mm::{DmaDirection, DmaStream, FrameAllocOptions, VmIo};

use crate::BLOCK_SIZE;

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
    dma_streams: Vec<DmaStream>,
    on_complete: Option<Box<dyn Fn(&BlockIo)>>,
}

impl BlockIo {
    pub fn new(operation: BlockOperation, block_num: usize) -> Self {
        let dma_streams = (0..block_num)
            .map(|_| {
                DmaStream::map(
                    FrameAllocOptions::new().alloc_segment(1).unwrap().into(),
                    DmaDirection::Bidirectional,
                    false,
                )
                .unwrap()
            })
            .collect::<Vec<_>>();

        Self {
            inner: BlockIoInner {
                operation,
                dma_streams,
                on_complete: None,
            },
        }
    }

    pub fn on_complete(&mut self, callback: Box<dyn Fn(&Self)>) {
        self.inner.on_complete = Some(callback);
    }

    pub fn read(&self, buf: &mut [u8]) {
        for (id, stream) in self.inner.dma_streams.iter().enumerate() {
            stream
                .read_bytes(
                    id * BLOCK_SIZE,
                    &mut buf[id * BLOCK_SIZE..(id + 1) * BLOCK_SIZE],
                )
                .unwrap();
            stream.sync(0..BLOCK_SIZE).unwrap();
        }
    }

    pub fn write(&self, buf: &[u8]) {
        for (id, stream) in self.inner.dma_streams.iter().enumerate() {
            stream
                .write_bytes(
                    id * BLOCK_SIZE,
                    &buf[id * BLOCK_SIZE..(id + 1) * BLOCK_SIZE],
                )
                .unwrap();
            stream.sync(0..BLOCK_SIZE).unwrap();
        }
    }
}

impl BlockIo {
    pub fn complete(&self) {
        if let Some(callback) = self.inner.on_complete.as_ref() {
            callback(self);
        }
    }

    pub fn dma_stream(&self) -> &[DmaStream] {
        &self.inner.dma_streams
    }

    pub fn operation(&self) -> BlockOperation {
        self.inner.operation
    }
}
