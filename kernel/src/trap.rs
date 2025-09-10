use zodiac::{
    hal::{
        context::{CpuException, PageFaultErrorCode, TrapFrame},
        cpu::Cpu,
    },
    mem::{MMUFlags, PhysicalMemoryAllocOptions},
    task::Task,
};

use crate::task::ThreadData;

pub fn init() {}

pub fn user_page_fault_handler(frame: &mut TrapFrame, cpu_exception: CpuException) {
    let CpuException::PageFault(flags, address) = cpu_exception else {
        unreachable!()
    };

    let thread = Task::current();
    let data = thread.data().downcast_ref::<ThreadData>().unwrap();
    let process = data.process.upgrade().unwrap();

    if process.is_child_process()
        && flags.contains(
            PageFaultErrorCode::PROTECTION_VIOLATION | PageFaultErrorCode::CAUSED_BY_WRITE,
        )
    {
        let (physical_memory, flags, page_size) = data.vm_space.query(address).unwrap();

        let mut new_physical_memory = PhysicalMemoryAllocOptions::default()
            .count(1)
            .page_size(page_size)
            .allocate()
            .unwrap();

        new_physical_memory
            .as_mut_slice(0)
            .unwrap()
            .copy_from_slice(physical_memory.as_slice(0).unwrap());

        let mut cursor = data.vm_space.cursor(address, page_size).unwrap();
        cursor.unmap(page_size as usize).unwrap();
        cursor
            .map(&physical_memory, flags | MMUFlags::WRITE)
            .unwrap();
    } else {
        {
            let mut unused = data.unused_region.write();
            let mut id = None;
            for (index, (region, flags, page_size)) in unused.iter().enumerate() {
                log::info!("{:x} {:x}", region.start_address(), region.len());
                if region.contains(address) {
                    log::info!("found");
                    let physical_memory = PhysicalMemoryAllocOptions::default()
                        .count(
                            page_size.align_up(
                                region.len() + region.start_address()
                                    - page_size.align_down(region.start_address()),
                            ) / *page_size as usize,
                        )
                        .page_size(*page_size)
                        .allocate()
                        .unwrap();

                    data.vm_space
                        .cursor(page_size.align_down(region.start_address()), *page_size)
                        .unwrap()
                        .map(&physical_memory, *flags)
                        .unwrap();

                    id = Some(index);
                }
            }

            if let Some(id) = id {
                unused.remove(id);
                return;
            }
        }

        log::warn!(
            "{:x?} on {:?} frame: {:#x?}",
            cpu_exception,
            Cpu::current(),
            frame
        );
        let (_, flags, _) = data.vm_space.query(address).unwrap();
        log::warn!("Fault address flags: {:?}", flags);
        process.exit();
    }
}
