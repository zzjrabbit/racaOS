#![allow(dead_code)]

use alloc::{sync::Arc, vec::Vec};
use errors::{Errno, Result};
use mostd::{
    io::IoMem,
    mem::{PhysicalMemory, PhysicalMemoryAllocOptions, VirtualAddress},
};
use spin::RwLock;

use crate::PAGE_SIZE;

mod rw;

#[derive(Debug, Clone)]
pub struct Vmo {
    inner: Arc<VmoInner>,
}

type PhysicalMemoryRef = Arc<PhysicalMemory>;

#[derive(Debug)]
enum VmoInner {
    Ram {
        frames: RwLock<Vec<Option<PhysicalMemoryRef>>>,
    },
    IoMem {
        iomem: Arc<IoMem>,
        offset: usize,
    },
}

impl Vmo {
    pub fn allocate_ram(count: usize) -> Result<Self> {
        Ok(Self {
            inner: Arc::new(VmoInner::Ram {
                frames: RwLock::new(alloc::vec![None; count]),
            }),
        })
    }

    pub fn acquire_iomem(address: VirtualAddress, length: usize) -> Result<Self> {
        Ok(Self {
            inner: Arc::new(VmoInner::IoMem {
                iomem: IoMem::acquire(address..address + length)?,
                offset: address % PAGE_SIZE,
            }),
        })
    }

    pub fn deep_clone(&self) -> Result<Self> {
        match self.inner.as_ref() {
            VmoInner::Ram { frames } => {
                let mut new_frames = alloc::vec![None; frames.read().len()];
                for (i, dest) in new_frames.iter_mut().enumerate() {
                    let source = frames.read()[i].clone();
                    if let Some(source) = source {
                        let frame = Arc::new(PhysicalMemoryAllocOptions::new().allocate()?);

                        let mut buffer = alloc::vec![0u8; PAGE_SIZE];
                        source.reader(0, buffer.len()).read_bytes(&mut buffer)?;
                        frame.writer(0, buffer.len()).write_bytes(&buffer)?;

                        *dest = Some(frame.clone().into());
                    }
                }
                Ok(Self {
                    inner: Arc::new(VmoInner::Ram {
                        frames: RwLock::new(new_frames),
                    }),
                })
            }
            VmoInner::IoMem { .. } => {
                Err(Errno::EACCES.with_message("Attempting to deep clone IoMem."))
            }
        }
    }
}

impl Vmo {
    pub(super) fn into_ram(&self, offset: usize) -> Result<Option<(usize, PhysicalMemoryRef)>> {
        match self.inner.as_ref() {
            VmoInner::Ram { frames } => {
                let id = offset / PAGE_SIZE;
                let page_offset = offset % PAGE_SIZE;

                let frame = frames.read()[id].clone();
                match frame {
                    Some(frame) => Ok(Some((page_offset, frame))),
                    None => {
                        let frame = Arc::new(PhysicalMemoryAllocOptions::new().allocate()?);
                        frames.write()[id] = Some(frame.clone());
                        Ok(Some((page_offset, frame)))
                    }
                }
            }
            VmoInner::IoMem { .. } => Ok(None),
        }
    }

    pub(super) fn into_iomem(&self) -> Option<(Arc<IoMem>, usize)> {
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

impl Vmo {
    pub fn split(&self, id: usize) -> Result<Self> {
        match self.inner.as_ref() {
            VmoInner::Ram { frames } => {
                let mut frames = frames.write();
                let new_frames = frames.split_off(id);
                Ok(Self {
                    inner: Arc::new(VmoInner::Ram {
                        frames: RwLock::new(new_frames),
                    }),
                })
            }
            VmoInner::IoMem { .. } => Err(Errno::EINVAL.no_message()),
        }
    }
}
