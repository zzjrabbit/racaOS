use zodiac::{
    mem::{MMUFlags, PhysicalMemoryAllocOptions},
    task::Task,
};

use crate::task::ThreadData;

use super::*;

pub fn mmap(
    address: usize,
    len: usize,
    _protection: usize,
    _flags: usize,
    _fd: usize,
    _offset: usize,
) -> SyscallResult {
    let thread = Task::current();
    let data = thread.data().downcast_ref::<ThreadData>().unwrap();

    let (address, mut cursor, page_size) = if address == 0 {
        data.allocate(len, false)
    } else {
        data.allocate_at(address, len, false)
    }?;

    let physical_memory = PhysicalMemoryAllocOptions::default()
        .count(
            page_size.align_up(len + address - page_size.align_down(address)) / page_size as usize,
        )
        .page_size(page_size)
        .allocate()
        .unwrap();

    cursor.map(&physical_memory, MMUFlags::USER_CODE | MMUFlags::USER_DATA)?;

    Ok(address as isize)
}
