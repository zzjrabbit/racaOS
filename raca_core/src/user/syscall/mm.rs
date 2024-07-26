use core::alloc::Layout;

use crate::user::get_current_process;

pub fn malloc(size: usize, align: usize) -> usize {
    let layout = Layout::from_size_align(size, align);
    if let Ok(layout) = layout {
        let process = get_current_process();
        let addr = process.write().heap.allocate(layout);
        if let Some(addr) = addr {
            addr as usize
        } else {
            0
        }
    } else {
        0
    }
}

pub fn free(addr: usize, size: usize, align: usize) -> usize {
    let layout = Layout::from_size_align(size, align);
    if let Ok(layout) = layout {
        let process = get_current_process();
        process.write().heap.deallocate(addr as u64, layout);
    }
    0
}
