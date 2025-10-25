use ostd::{mm::Vaddr, task::Task};

use crate::{
    AsThread, UserThreadData,
    syscall::{SyscallResult, timespec_t},
};

pub fn clock_gettime(_clock_id: i32, timespec_addr: Vaddr) -> SyscallResult {
    let time = time::DateTime::default().unix_timestamp();
    let timespec = timespec_t::from(time);

    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();
    data.memory_info()
        .vmar()
        .write_val(timespec_addr, &timespec)?;

    Ok(0)
}
