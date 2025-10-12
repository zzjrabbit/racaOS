#![allow(dead_code)]

use alloc::{sync::Arc, vec::Vec};
use ostd::{
    io::IoMem,
    mm::{FrameAllocOptions, HasSize, UFrame, Vaddr, VmIo, PAGE_SIZE},
    sync::RwMutex,
    Error,
};

mod rw;

#[derive(Debug, Clone)]
pub struct Vmo {
    inner: Arc<VmoInner>,
}

#[derive(Debug)]
enum VmoInner {
    Ram {
        frames: RwMutex<Vec<Option<UFrame>>>,
    },
    IoMem {
        iomem: IoMem,
        offset: usize,
    },
}

impl Vmo {
    pub fn allocate_ram(count: usize) -> Result<Self, Error> {
        Ok(Self {
            inner: Arc::new(VmoInner::Ram {
                frames: RwMutex::new(alloc::vec![None; count]),
            }),
        })
    }

    pub fn acquire_iomem(address: Vaddr, length: usize) -> Result<Self, Error> {
        Ok(Self {
            inner: Arc::new(VmoInner::IoMem {
                iomem: IoMem::acquire(address..address + length)?,
                offset: address % PAGE_SIZE,
            }),
        })
    }

    pub fn deep_clone(&self) -> Result<Self, Error> {
        match self.inner.as_ref() {
            VmoInner::Ram { frames } => {
                let mut new_frames = alloc::vec![None; frames.read().len()];
                for (i, dest) in new_frames.iter_mut().enumerate() {
                    let source = frames.read()[i].clone();
                    if let Some(source) = source {
                        let frame = FrameAllocOptions::new().alloc_frame()?;

                        let mut buffer = alloc::vec![0u8; PAGE_SIZE];
                        source.read_bytes(0, &mut buffer)?;
                        frame.write_bytes(0, &buffer)?;

                        *dest = Some(frame.clone().into());
                    }
                }
                Ok(Self {
                    inner: Arc::new(VmoInner::Ram {
                        frames: RwMutex::new(new_frames),
                    }),
                })
            }
            VmoInner::IoMem { .. } => Err(Error::AccessDenied),
        }
    }
}

impl Vmo {
    pub(super) fn into_ram(&self, offset: usize) -> Result<Option<(usize, UFrame)>, Error> {
        match self.inner.as_ref() {
            VmoInner::Ram { frames } => {
                let id = offset / PAGE_SIZE;
                let page_offset = offset % PAGE_SIZE;

                let frame = frames.read()[id].clone();
                match frame {
                    Some(frame) => Ok(Some((page_offset, frame))),
                    None => {
                        let frame = FrameAllocOptions::new().alloc_frame()?;
                        frames.write()[id] = Some(frame.clone().into());
                        Ok(Some((page_offset, frame.into())))
                    }
                }
            }
            VmoInner::IoMem { .. } => Ok(None),
        }
    }

    pub(super) fn into_iomem(&self) -> Option<(IoMem, usize)> {
        match self.inner.as_ref() {
            VmoInner::Ram { .. } => None,
            VmoInner::IoMem { iomem, offset } => Some((iomem.clone(), *offset)),
        }
    }

    pub(super) fn commited(&self, id: usize) -> bool {
        match self.inner.as_ref() {
            VmoInner::Ram { frames } => frames.read()[id].is_some(),
            VmoInner::IoMem { .. } => true,
        }
    }
}

impl Vmo {
    pub fn len(&self) -> usize {
        match self.inner.as_ref() {
            VmoInner::Ram { frames } => frames.read().len() * PAGE_SIZE,
            VmoInner::IoMem { iomem, .. } => iomem.size(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self.inner.as_ref() {
            VmoInner::Ram { frames } => frames.read().is_empty(),
            VmoInner::IoMem { iomem, .. } => iomem.size() == 0,
        }
    }

    pub fn is_iomem(&self) -> bool {
        match self.inner.as_ref() {
            VmoInner::Ram { .. } => false,
            VmoInner::IoMem { .. } => true,
        }
    }
}
