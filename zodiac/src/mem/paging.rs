use loongarch64::{
    PhysAddr, VirtAddr,
    registers::{PgdHigh, PgdLow, init_pwc},
    structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PageProperty, Size4KiB},
};

use crate::mem::{FRAME_ALLOCATOR, PHYSICAL_MEMORY_OFFSET, convert_physical_to_virtual};

pub fn init() {
    init_pwc();

    let physical_offset = *PHYSICAL_MEMORY_OFFSET;

    let pgdl = convert_physical_to_virtual(PhysAddr::new(PgdLow.read()));
    let pgdh = convert_physical_to_virtual(PhysAddr::new(PgdHigh.read()));

    let lower_half = unsafe { &mut *pgdl.as_mut_ptr() };
    let higher_half = unsafe { &mut *pgdh.as_mut_ptr() };

    let mut page_table =
        unsafe { OffsetPageTable::new(lower_half, higher_half, physical_offset as u64) };

    let frame = FRAME_ALLOCATOR.lock().allocate_frame().unwrap();

    unsafe {
        page_table
            .map_to(
                Page::<Size4KiB>::containing_address(VirtAddr::new(0xffff_c000_0000_0000)),
                frame,
                PageProperty::kernel_data(),
                &mut *FRAME_ALLOCATOR.lock(),
            )
            .unwrap()
            .flush();
    }
}
