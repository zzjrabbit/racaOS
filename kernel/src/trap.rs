use zodiac::{
    hal::{
        context::{CpuException, PageFaultErrorCode, TrapFrame},
        cpu::Cpu,
        trap::set_user_page_fault_handler,
    },
    mem::{MMUFlags, PhysicalMemoryAllocOptions},
};

pub fn init() {
    set_user_page_fault_handler(user_page_fault_handler);
}

fn user_page_fault_handler(frame: &mut TrapFrame, cpu_exception: CpuException) {
    let CpuException::PageFault(flags, address) = cpu_exception else {
        unreachable!()
    };

    let process = zodiac::task::Thread::current().process().unwrap();

    if process.is_child_process()
        && flags.contains(
            PageFaultErrorCode::PROTECTION_VIOLATION | PageFaultErrorCode::CAUSED_BY_WRITE,
        )
    {
        let (physical_memory, flags, page_size) = process.vm_space().query(address).unwrap();

        let new_physical_memory = PhysicalMemoryAllocOptions::default()
            .count(1)
            .page_size(page_size)
            .allocate()
            .unwrap();

        new_physical_memory
            .as_mut_slice(0)
            .unwrap()
            .copy_from_slice(physical_memory.as_slice(0).unwrap());

        let mut cursor = process.vm_space().cursor(address, page_size).unwrap();
        cursor.unmap(page_size as usize).unwrap();
        cursor
            .map(&physical_memory, flags | MMUFlags::WRITE)
            .unwrap();
    } else {
        log::warn!(
            "{:?} on {:?} frame: {:?}",
            cpu_exception,
            Cpu::current(),
            frame
        );
        process.kill();
    }
}
