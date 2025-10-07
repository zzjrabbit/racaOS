use alloc::sync::Arc;
use ostd::{Pod, mm::{CachePolicy, FrameAllocOptions, PAGE_SIZE, PageFlags, PageProperty, VmSpace}, task::disable_preempt};

use crate::{mem::VmReadWrite, task::MemoryInfo};

pub struct UserStack {
    vm_space: Arc<VmSpace>,
    stack_pointer: usize,
}

const USER_STACK_SIZE: usize = 8 * 1024 * 1024;

impl UserStack {
    pub fn new(memory_info: &MemoryInfo) -> Self {
        let stack_region = memory_info.allocate(USER_STACK_SIZE).unwrap();
        let disable_preempt_guard = disable_preempt();
        let vm_space = memory_info.vm_space();

        let mut cursor = vm_space
            .cursor_mut(
                &disable_preempt_guard,
                &(stack_region.start_address()..stack_region.end_address()),
            )
            .unwrap();

        for _ in 0..USER_STACK_SIZE / PAGE_SIZE {
            let frame = FrameAllocOptions::new().alloc_frame().unwrap();

            let property = PageProperty::new_user(PageFlags::RW, CachePolicy::Writeback);
            cursor.map(frame.into(), property);
        }
        
        let stack_pointer = stack_region.end_address();

        Self {
            vm_space: memory_info.vm_space(),
            stack_pointer,
        }
    }
}

impl UserStack {
    pub fn push<T: Pod>(&mut self, value: T) -> usize {
        let value_ptr = self.stack_pointer - core::mem::size_of::<T>();
        self.vm_space.write_val(value_ptr, &value).unwrap();
        self.stack_pointer = value_ptr;
        value_ptr
    }
    
    pub fn push_a_lot<T: Pod>(&mut self, values: &[T]) -> usize {
        for value in values.iter().rev() {
            self.push(*value);
        }
        self.stack_pointer
    }
    
    pub fn push_zero_until_aligned(&mut self, alignment: usize) {
        let remainder = self.stack_pointer % alignment;
        self.push_a_lot(&alloc::vec![0u8; remainder]);
    }
    
    pub fn stack_pointer(&self) -> usize {
        self.stack_pointer
    }
}
