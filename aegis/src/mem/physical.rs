use alloc::{sync::Arc, vec::Vec};
use spin::RwLock;

use crate::{
    AegisError,
    PhyscialMemoryError,
    hal::mem::align_down_by_page_size,
    mem::{FRAME_ALLOCATOR, PageSize, PhysicalAddress},
};

pub struct PhysicalMemory {
    count: usize,
    page_size: PageSize,
    contiguous: bool, 
    inner: RwLock<PhysicalMemoryInner>,
}

struct PhysicalMemoryInner {
    commited: bool,
    start_address: Option<PhysicalAddress>,
    frames: Vec<PhysicalAddress>,
}

impl PhysicalMemory {
    pub fn new(count: usize, page_size: PageSize, contiguous: bool) -> Arc<Self> {
        Arc::new(Self {
            count,
            page_size,
            contiguous,
            inner: RwLock::new(PhysicalMemoryInner {
                commited: false,
                start_address: None,
                frames: Vec::new(),
            }),
        })
    }
}

impl PhysicalMemory {
    pub fn from_start_address(
        start_address: PhysicalAddress,
        count: usize,
        page_size: PageSize,
    ) -> Arc<Self> {
        Arc::new(Self {
            count,
            page_size,
            contiguous: true,
            inner: RwLock::new(PhysicalMemoryInner {
                commited: true,
                start_address: Some(start_address),
                frames: Vec::new(),
            }),
        })
    }

    pub fn containing_address(
        address: PhysicalAddress,
        count: usize,
        page_size: PageSize,
    ) -> Arc<Self> {
        let start_address = align_down_by_page_size(address);

        Self::from_start_address(start_address, count, page_size)
    }
}

impl PhysicalMemory {
    pub fn commited(&self) -> bool {
        self.inner.read().commited
    }

    pub fn commit(&self) -> Result<(), AegisError> {
        if self.commited() {
            return Ok(());
        }
        
        let one_frame_count = self.page_size as usize / PageSize::Size4K as usize;
        
        if self.contiguous() {
            if let Some(start_address) = FRAME_ALLOCATOR
                .lock()
                .allocate_frames(self.count * one_frame_count, one_frame_count)
            {
                self.inner.write().commited = true;
                self.inner.write().start_address = Some(start_address);
                Ok(())
            } else {
                Err(PhyscialMemoryError::AllocateFailed(self.count).into())
            }
        } else {
            let mut inner = self.inner.write();
            
            while inner.frames.len() <= self.count() {
                let Some(start_address) = FRAME_ALLOCATOR
                    .lock()
                    .allocate_frames(one_frame_count, one_frame_count)
                else {
                    return Err(PhyscialMemoryError::AllocateFailed(one_frame_count).into())
                };
                
                inner.frames.push(start_address);
            }
            self.inner.write().commited = true;
            Ok(())
        }
    }
}

impl PhysicalMemory {
    pub fn get_start_address_of_frame(&self, id: usize) -> Result<PhysicalAddress, AegisError> {
        if id >= self.count() {
            return Err(AegisError::InvalidArguments);
        }
        
        self.commit()?;
        
        if self.contiguous() {
            Ok(self.inner.read().start_address.unwrap() + (id * self.page_size as usize))
        } else {
            Ok(*self.inner.read().frames.get(id).unwrap())
        }
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn page_size(&self) -> PageSize {
        self.page_size
    }
    
    pub fn contiguous(&self) -> bool {
        self.contiguous
    }
}
