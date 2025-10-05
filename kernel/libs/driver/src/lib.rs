#![no_std]

extern crate alloc;

use core::marker::PhantomData;

use alloc::sync::Arc;
use ostd::{
    Pod,
    io::IoMem,
    mm::{DmaCoherent, FrameAllocOptions, HasDaddr, PAGE_SIZE, Paddr, VmIo, VmIoFill},
};

trait MmioInner {
    fn read_bytes(&self, offset: usize, buffer: &mut [u8]) -> ostd::Result<()>;
    fn write_bytes(&self, offset: usize, buffer: &[u8]) -> ostd::Result<()>;
}

impl<T: VmIo> MmioInner for T {
    fn read_bytes(&self, offset: usize, buffer: &mut [u8]) -> ostd::Result<()> {
        self.read_bytes(offset, buffer)
    }

    fn write_bytes(&self, offset: usize, buffer: &[u8]) -> ostd::Result<()> {
        self.write_bytes(offset, buffer)
    }
}

pub struct Mmio<T: Pod> {
    offset: usize,
    inner: Arc<dyn MmioInner>,
    _marker: PhantomData<T>,
}

impl<T: Pod> Mmio<T> {
    pub fn new(address: Paddr) -> ostd::Result<Self> {
        Ok(Self {
            offset: 0,
            inner: Arc::new(IoMem::acquire(address..address + size_of::<T>())?),
            _marker: PhantomData,
        })
    }

    pub fn new_from<I: VmIo + 'static>(inner: Arc<I>, offset: usize) -> Self {
        Self {
            offset,
            inner,
            _marker: PhantomData,
        }
    }
}

impl<T: Pod> Mmio<T> {
    pub fn read(&self) -> T {
        let mut buffer = alloc::vec![0; size_of::<T>()];
        self.inner.read_bytes(self.offset, &mut buffer).unwrap();
        T::from_bytes(&buffer)
    }

    pub fn write(&self, value: &T) {
        self.inner
            .write_bytes(self.offset, value.as_bytes())
            .unwrap();
    }
}

pub struct DmaList<T: Pod> {
    data: DmaCoherent,
    _marker: PhantomData<T>,
}

impl<T: Pod> DmaList<T> {
    pub fn new(length: usize) -> Self {
        let size = length * size_of::<T>();
        let page_count = size.div_ceil(PAGE_SIZE);

        let segment = FrameAllocOptions::new().alloc_segment(page_count).unwrap();

        let data = DmaCoherent::map(segment.into(), false).unwrap();
        data.fill_zeros(0, page_count * PAGE_SIZE).unwrap();

        Self {
            data,
            _marker: PhantomData,
        }
    }
}

impl<T: Pod> DmaList<T> {
    pub fn read(&self, index: usize) -> T {
        let offset = index * size_of::<T>();
        self.data.read_val(offset).unwrap()
    }

    pub fn write(&self, index: usize, value: &T) {
        let offset = index * size_of::<T>();
        self.data.write_val(offset, value).unwrap();
    }

    pub fn with_value<F, R>(&self, index: usize, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut value = self.read(index);
        let result = f(&mut value);
        self.write(index, &value);
        result
    }
}

impl<T: Pod> DmaList<T> {
    pub fn device_address(&self) -> usize {
        self.data.daddr()
    }

    pub fn device_address_of(&self, index: usize) -> usize {
        self.device_address() + index * size_of::<T>()
    }
}
