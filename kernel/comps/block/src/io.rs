use core::sync::atomic::{AtomicBool, Ordering};

use alloc::{sync::Arc, vec::Vec};
use ostd::{
    mm::{DmaDirection, DmaStream, FrameAllocOptions, VmIo},
    sync::WaitQueue,
};

use crate::{BLOCK_SIZE, BlockDevice, BlockDeviceError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BlockOperation {
    Read,
    Write,
}

pub struct BlockIo {
    inner: Arc<BlockIoInner>,
}

struct BlockIoInner {
    operation: BlockOperation,
    dma_streams: Vec<DmaStream>,
    complete: AtomicBool,
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
            inner: Arc::new(BlockIoInner {
                operation,
                dma_streams,
                complete: AtomicBool::new(false),
            }),
        }
    }

    pub fn read(&self, offset: usize, buffer: &mut [u8]) {
        let mut read: usize = 0;
        
        while read < buffer.len() {
            let current = offset + read;
            let block_offset = current % BLOCK_SIZE;
            let remaining = buffer.len() - read;
            let chunk_size = (BLOCK_SIZE - block_offset).min(remaining) as usize;

            let block_id = current / BLOCK_SIZE;
            let block = &self.inner.dma_streams[block_id];
            block.sync(0..BLOCK_SIZE).unwrap();
            block.read_bytes(block_offset, &mut buffer[read..read + chunk_size]).unwrap();

            read += chunk_size;
        }
    }

    pub fn write(&self, offset: usize, buffer: &[u8]) {
        let mut written: usize = 0;

        while written < buffer.len() {
            let current = offset + written;
            let block_offset = current % BLOCK_SIZE;
            let remaining = buffer.len() - written;
            let chunk_size = (BLOCK_SIZE - block_offset).min(remaining) as usize;

            let block_id = current / BLOCK_SIZE;
            let block = &self.inner.dma_streams[block_id];
            block.write_bytes(block_offset, &buffer[written..written + chunk_size]).unwrap();
            block.sync(0..BLOCK_SIZE).unwrap();

            written += chunk_size;
        }
    }

    pub fn commit(
        &self,
        block_offset: u64,
        device: Arc<dyn BlockDevice>,
    ) -> Result<BlockIoWaiter, BlockDeviceError> {
        device
            .commit_io(
                block_offset,
                Self {
                    inner: self.inner.clone(),
                },
            )
            .map(|_| BlockIoWaiter::new(self.inner.clone()))
    }
}

impl BlockIo {
    pub fn complete(&self) {
        self.inner.complete.store(true, Ordering::SeqCst);
    }

    pub fn dma_stream(&self) -> &[DmaStream] {
        &self.inner.dma_streams
    }

    pub fn operation(&self) -> BlockOperation {
        self.inner.operation
    }
}

pub struct BlockIoWaiter {
    bio: Arc<BlockIoInner>,
    wait_queue: WaitQueue,
}

impl BlockIoWaiter {
    fn new(bio: Arc<BlockIoInner>) -> Self {
        Self {
            bio,
            wait_queue: WaitQueue::new(),
        }
    }
}

impl BlockIoWaiter {
    pub fn wait(&self) {
        self.wait_queue
            .wait_until(|| self.bio.complete.load(Ordering::SeqCst).then_some(()));
    }
}
