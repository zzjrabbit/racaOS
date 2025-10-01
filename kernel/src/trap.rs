use ostd::{
    arch::{
        cpu::context::{CpuException, PageFaultErrorCode, RawPageFaultInfo},
        trap::inject_user_page_fault_handler,
    },
    cpu::CpuId,
    mm::{vm_space::VmQueriedItem, FrameAllocOptions, PageFlags, PageProperty, VmIo, PAGE_SIZE},
    task::{disable_preempt, Task},
};

use crate::{
    mem::{align_down_by_page_size, align_up_by_page_size},
    task::ThreadData,
};

pub fn init() {
    inject_user_page_fault_handler(user_page_fault_handler);
}

pub fn user_page_fault_handler(cpu_exception: &CpuException) -> Result<(), ()> {
    let CpuException::PageFault(RawPageFaultInfo { error_code, addr }) = cpu_exception else {
        unreachable!()
    };

    let thread = Task::current().unwrap();
    let data = thread.data().downcast_ref::<ThreadData>().unwrap();
    let process = data.process.upgrade().unwrap();

    if process.is_child_process()
        && error_code.contains(PageFaultErrorCode::PROTECTION | PageFaultErrorCode::WRITE)
    {
        let disable_preempt_guard = disable_preempt();
        let (_, item) = data
            .vm_space
            .cursor_mut(&disable_preempt_guard, &(*addr..*addr + 1))
            .unwrap()
            .query()
            .unwrap();
        let VmQueriedItem::MappedRam { frame, prop } = item.unwrap() else {
            unreachable!()
        };

        let new_physical_memory = FrameAllocOptions::default().alloc_frame().unwrap();

        let mut buffer = [0u8; PAGE_SIZE];
        frame.read_bytes(0, &mut buffer).unwrap();

        new_physical_memory.write_bytes(0, &buffer).unwrap();

        let mut cursor = data
            .vm_space
            .cursor_mut(&disable_preempt_guard, &(*addr..*addr + PAGE_SIZE))
            .unwrap();
        cursor.unmap(PAGE_SIZE);

        cursor.jump(*addr).unwrap();
        cursor.map(
            new_physical_memory.into(),
            PageProperty::new_user(prop.flags | PageFlags::W, prop.cache),
        );
    } else {
        {
            let disable_preempt_guard = disable_preempt();

            let mut unused = data.unused_region.write();
            let mut id = None;
            for (index, (region, flags)) in unused.iter().enumerate() {
                if region.contains(*addr) {
                    let page_count = align_up_by_page_size(
                        region.len() + region.start_address()
                            - align_down_by_page_size(region.start_address()),
                    ) / PAGE_SIZE;

                    for page_id in 0..page_count {
                        let address = region.start_address() + page_id * PAGE_SIZE;

                        let frame = FrameAllocOptions::new().alloc_frame().unwrap();

                        data.vm_space
                            .cursor_mut(&disable_preempt_guard, &(address..address + PAGE_SIZE))
                            .unwrap()
                            .map(frame.into(), *flags);
                    }

                    id = Some(index);
                }
            }

            if let Some(id) = id {
                unused.remove(id);
                return Ok(());
            }
        }

        log::warn!("{:x?} on {:?}", cpu_exception, CpuId::current_racy(),);
        process.exit();
    }

    Ok(())
}
