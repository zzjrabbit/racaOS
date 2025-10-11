use ostd::{
    arch::{
        cpu::context::{CpuException, PageFaultErrorCode, RawPageFaultInfo},
        trap::inject_user_page_fault_handler,
    },
    mm::PageFlags,
    task::Task,
};

use crate::task::{AsThread, UserThreadData};

pub fn init() {
    inject_user_page_fault_handler(user_page_fault_handler);
}

pub fn user_page_fault_handler(cpu_exception: &CpuException) -> Result<(), ()> {
    let CpuException::PageFault(RawPageFaultInfo { error_code, addr }) = cpu_exception else {
        unreachable!()
    };

    let thread = Task::current().unwrap();
    let data = thread.direct_downcast::<UserThreadData>().unwrap();
    let process = data.process.upgrade().unwrap();

    let required_flags = {
        let mut flags = PageFlags::empty();
        if error_code.contains(PageFaultErrorCode::WRITE) {
            flags |= PageFlags::W;
        }
        if error_code.contains(PageFaultErrorCode::INSTRUCTION) {
            flags |= PageFlags::X;
        }
        flags
    };

    if !data
        .memory_info()
        .vmar()
        .handle_page_fault(*addr, required_flags)
        .map_err(|_| ())?
    {
        log::error!("Unhandled page fault: {:x?}", cpu_exception);
        process.exit(-1);
    }

    Ok(())
}
