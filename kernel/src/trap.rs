use ostd::{
    arch::{
        cpu::context::{CpuException, RawPageFaultInfo},
        trap::inject_user_page_fault_handler,
    },
    task::Task,
};

use crate::task::{AsThread, UserThreadData};

pub fn init() {
    inject_user_page_fault_handler(user_page_fault_handler);
}

pub fn user_page_fault_handler(cpu_exception: &CpuException) -> Result<(), ()> {
    let CpuException::PageFault(RawPageFaultInfo {
        error_code: _,
        addr,
    }) = cpu_exception
    else {
        unreachable!()
    };

    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();
    let process = data.process.upgrade().unwrap();

    if !data
        .memory_info()
        .vmar()
        .handle_page_fault(*addr)
        .map_err(|_| ())?
    {
        log::error!("Unhandled page fault: {:x?}", cpu_exception);
        process.exit(-1);
    }

    Ok(())
}
