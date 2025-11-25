use core::fmt::Display;

use alloc::fmt;
use bit_field::BitField;
use humansize::{BINARY, format_size};
use limine::{memory_map::EntryType, response::MemoryMapResponse};

use crate::mem::{PhysicalAddress, convert_physical_to_virtual};

pub struct Bitmap(&'static mut [usize]);

#[allow(dead_code)]
impl Bitmap {
    const BITS: usize = usize::BITS as usize;

    pub fn new(inner: &'static mut [usize]) -> Self {
        inner.fill(0);
        Self(inner)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len() * Self::BITS
    }

    #[inline]
    pub fn get(&self, index: usize) -> bool {
        let byte = self.0[index / Self::BITS];
        byte.get_bit(index % Self::BITS)
    }

    #[inline]
    pub fn set(&mut self, index: usize, value: bool) {
        let byte = &mut self.0[index / Self::BITS];
        byte.set_bit(index % Self::BITS, value);
    }
}

impl Bitmap {
    pub fn set_range(&mut self, start: usize, end: usize, value: bool) {
        if start >= end || start >= self.len() {
            return;
        }

        let start_byte = start.div_ceil(Self::BITS);
        let end_byte = end / Self::BITS;

        (start..(start_byte * Self::BITS).min(end)).for_each(|i| self.set(i, value));

        if start_byte > end_byte {
            return;
        }

        if start_byte <= end_byte {
            let fill_value = if value { usize::MAX } else { 0 };
            self.0[start_byte..end_byte].fill(fill_value);
        }

        ((end_byte * Self::BITS).max(start)..end).for_each(|i| self.set(i, value));
    }

    pub fn find_range(&mut self, length: usize, value: bool) -> Option<usize> {
        let byte_match = if value { usize::MAX } else { 0 };
        let (mut count, mut start_index) = (0, 0);

        for (index, &byte) in self.0.iter().enumerate() {
            match byte {
                b if b == !byte_match => count = 0,
                b if b == byte_match => {
                    if length < Self::BITS {
                        return Some(index * Self::BITS);
                    }
                    if count == 0 {
                        start_index = index * Self::BITS;
                    }
                    count += Self::BITS;
                    if count >= length {
                        return Some(start_index);
                    }
                }
                _ => {
                    for bit_index in 0..Self::BITS {
                        if byte.get_bit(bit_index) == value {
                            if count == 0 {
                                start_index = index * Self::BITS + bit_index;
                            }
                            count += 1;
                            if count == length {
                                return Some(start_index);
                            }
                        } else {
                            count = 0;
                        }
                    }
                }
            }
        }

        None
    }
}

pub struct BitmapFrameAllocator {
    bitmap: Bitmap,
    origin_frames: usize,
    usable_frames: usize,
}

impl BitmapFrameAllocator {
    #[inline]
    fn used_bytes(&self) -> usize {
        (self.origin_frames - self.usable_frames) * 4096
    }

    #[inline]
    fn total_bytes(&self) -> usize {
        self.origin_frames * 4096
    }
}

impl Display for BitmapFrameAllocator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} used, {} total",
            format_size(self.used_bytes(), BINARY),
            format_size(self.total_bytes(), BINARY)
        )
    }
}

impl BitmapFrameAllocator {
    pub fn init(memory_map: &MemoryMapResponse) -> Self {
        let memory_size = memory_map
            .entries()
            .last()
            .map(|region| region.base + region.length)
            .expect("No memory regions found");

        let bitmap_size = (memory_size / 4096).div_ceil(8) as usize;

        let usable_regions = memory_map
            .entries()
            .iter()
            .filter(|region| region.entry_type == EntryType::USABLE);

        let bitmap_address = usable_regions
            .clone()
            .find(|region| region.length >= bitmap_size as u64)
            .map(|region| region.base)
            .expect("No suitable memory region for bitmap");

        let bitmap_buffer = unsafe {
            let physical_address = bitmap_address as PhysicalAddress;
            let virtual_address = convert_physical_to_virtual(physical_address);
            let bitmap_inner_size = bitmap_size / size_of::<usize>();
            core::slice::from_raw_parts_mut(virtual_address as *mut usize, bitmap_inner_size)
        };

        let mut bitmap = Bitmap::new(bitmap_buffer);
        let mut origin_frames = 0;

        for region in usable_regions {
            let start_page_index = (region.base / 4096) as usize;
            let frame_count = (region.length / 4096) as usize;

            origin_frames += frame_count;
            bitmap.set_range(start_page_index, start_page_index + frame_count, true);
        }

        let bitmap_frame_start = (bitmap_address / 4096) as usize;
        let bitmap_frame_count = bitmap_size.div_ceil(4096);
        let bitmap_frame_end = bitmap_frame_start + bitmap_frame_count;

        let usable_frames = origin_frames - bitmap_frame_count;
        bitmap.set_range(bitmap_frame_start, bitmap_frame_end, false);

        BitmapFrameAllocator {
            bitmap,
            origin_frames,
            usable_frames,
        }
    }

    pub fn allocate_frames(&mut self, count: usize) -> Option<PhysicalAddress> {
        let index = self
            .bitmap
            .find_range(count, true)
            .expect("No more usable frames!");

        self.bitmap.set_range(index, index + count, false);
        self.usable_frames -= count;

        let address = index * 4096;
        Some(address)
    }

    pub fn deallocate_frames(&mut self, address: PhysicalAddress, count: usize) {
        let index = address / 4096;
        self.bitmap.set_range(index, index + count, true);
        self.usable_frames += count;
    }
}
