/*
* @file    :   heap.rs
* @time    :   2024/04/14 08:45:00
* @author  :   zzjcarrot
*/

use crate::{memory::FRAME_ALLOCATOR, ref_to_mut, task::Process};
use alloc::{sync::Weak, vec::Vec};
use core::alloc::Layout;
use spin::{Mutex, RwLock};
use x86_64::{
    structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags},
    VirtAddr,
};

pub enum HeapType {
    Kernel,
    User,
}

pub const HEAP_START: u64 = 20 * 1024 * 1024 * 1024 * 1024; // 20TB(用户程序空间18TB~20TB)
pub const USER_HEAP_INIT_SIZE: usize = 32 * 1024; // 32KB

pub struct Heap(Mutex<Vec<(u64, usize)>>);

impl Heap {
    pub fn new() -> Self {
        Self(Mutex::new(Vec::new()))
    }

    pub fn init(&self, start: usize, size: usize) {
        let mut list = self.0.lock();
        list.push((start as u64, size));
    }

    pub fn add(&self, start: usize, size: usize) {
        let mut list = self.0.lock();
        list.push((start as u64, size));
    }

    pub fn alloc(&self, layout: Layout) -> Option<u64> {
        let mut list = self.0.lock();
        for idx in 0..list.len() {
            let (start, size) = list[idx];
            if start % layout.align() as u64 == 0 && size > layout.size() {
                let ptr = start as *mut u8;
                list[idx].0 += layout.size() as u64;
                list[idx].1 -= layout.size();
                return Some(ptr as u64);
            } else if start % layout.align() as u64 == 0 && size == layout.size() {
                let ptr = start as *mut u8;
                list.remove(idx);
                return Some(ptr as u64);
            }
        }
        None
    }

    pub fn dealloc(&self, ptr: u64, layout: Layout) {
        let mut list = self.0.lock();
        list.push((ptr as u64, layout.size()));
    }
}

pub struct ProcessHeap {
    heap_type: HeapType,
    size: usize,
    usable_size: usize,
    allocator: Heap,
    process: Option<Weak<RwLock<Process>>>,
}

impl ProcessHeap {
    pub fn new(heap_type: HeapType) -> Self {
        let size = match heap_type {
            HeapType::Kernel => 0,
            HeapType::User => USER_HEAP_INIT_SIZE,
        };
        let allocator = Heap::new();
        if size > 0 {
            allocator.init(HEAP_START as usize, size);
        }

        Self {
            heap_type,
            size,
            usable_size: size,
            allocator,
            process: None,
        }
    }

    pub fn init(&self, process: Weak<RwLock<Process>>) {
        match self.heap_type {
            HeapType::User => {
                ref_to_mut(self).process = Some(process.clone());
                let mut frame_allocator = FRAME_ALLOCATOR.lock();
                for page in 0..USER_HEAP_INIT_SIZE / 4096 {
                    let frame = frame_allocator.allocate_frame().unwrap();
                    let page =
                        Page::containing_address(VirtAddr::new(HEAP_START + page as u64 * 4096));
                    let flags = PageTableFlags::PRESENT
                        | PageTableFlags::WRITABLE
                        | PageTableFlags::USER_ACCESSIBLE;
                    unsafe {
                        ref_to_mut(&*process.upgrade().unwrap().read())
                            .page_table
                            .map_to(page, frame, flags, &mut *frame_allocator)
                            .unwrap()
                            .flush();
                    }
                }
            }

            _ => {}
        }
    }

    pub fn allocate(&mut self, layout: Layout) -> Option<u64> {
        match self.heap_type {
            HeapType::Kernel => {
                panic!("Don't use process heaps in kernel mode! Use kernel heap instead!")
            }
            _ => {}
        }
        if self.usable_size < layout.size() {
            let size = layout.size() * 2;
            let page_cnt = (size + 4095) / 4096;
            self.allocator.add(HEAP_START as usize + self.size, size);
            log::info!("need {}", size);
            for _ in 0..page_cnt {
                let frame = FRAME_ALLOCATOR.lock().allocate_frame().unwrap();
                let page = Page::containing_address(VirtAddr::new(HEAP_START + self.size as u64));
                let flags = PageTableFlags::PRESENT
                    | PageTableFlags::WRITABLE
                    | PageTableFlags::USER_ACCESSIBLE;
                unsafe {
                    self.process
                        .as_ref()
                        .unwrap()
                        .upgrade()
                        .unwrap()
                        .write()
                        .page_table
                        .map_to(page, frame, flags, &mut *FRAME_ALLOCATOR.lock())
                        .unwrap()
                        .flush();
                }

                /*KERNEL_PAGE_TABLE
                .try_get()
                .unwrap()
                .lock()
                .unmap(page)
                .unwrap()
                .1
                .flush();*/

                self.size += 4096;
                self.usable_size += 4096;
            }
        }

        let ptr = self.allocator.alloc(layout).unwrap();
        self.usable_size -= layout.size();
        Some(ptr as u64)
    }

    pub fn deallocate(&mut self, ptr: u64, layout: Layout) {
        match self.heap_type {
            HeapType::Kernel => panic!("Don't use process heaps in kernel mode!"),
            _ => {}
        }
        self.allocator.dealloc(ptr, layout);
        self.usable_size += layout.size();
    }
}
