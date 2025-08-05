use zodiac::mem::{MMUFlags, PhysicalMemoryAllocOptions};

use super::*;

pub fn mmap(
    address: usize,
    len: usize,
    _protection: usize,
    _flags: usize,
    _fd: usize,
    _offset: usize,
) -> SyscallResult {
    let current_process = Process::current();

    let (address, mut cursor, page_size) = if address == 0 {
        current_process.allocate(len, false)
    } else {
        current_process.allocate_at(address, len, false)
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
