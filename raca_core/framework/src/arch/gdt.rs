use core::ptr::addr_of;

use alloc::{boxed::Box, vec};
use conquer_once::spin::Lazy;
use spin::Mutex;
use x86_64::{
    instructions::{
        segmentation::{CS, DS, ES, FS, GS, SS},
        tables::load_tss,
    },
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

use crate::arch::smp::{BSP_ID, CPUS};

const STACK_SIZE: usize = 4096;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const PAGE_FAULT_IST_INDEX: u16 = 1;

macro_rules! tss {
    ($stack:expr) => {{
        let mut tss = TaskStateSegment::new();
        tss.privilege_stack_table[0] = {
            let stack_start = VirtAddr::from_ptr($stack);
            stack_start + STACK_SIZE as u64
        };
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            let stack_start = VirtAddr::from_ptr($stack);
            stack_start + STACK_SIZE as u64
        };
        tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] = {
            let stack_start = VirtAddr::from_ptr($stack);
            stack_start + STACK_SIZE as u64
        };
        tss
    }};
}

pub struct Selectors {
    pub kernel_code: SegmentSelector,
    pub kernel_data: SegmentSelector,
    pub user_code: SegmentSelector,
    pub user_data: SegmentSelector,
    pub tss: SegmentSelector,
}

impl Selectors {
    pub fn get_kernel_segments() -> (SegmentSelector, SegmentSelector) {
        let mut cpus = CPUS.lock();
        (cpus.current_cpu().selectors.kernel_code, cpus.current_cpu().selectors.kernel_data)
    }
    pub fn get_user_segments() -> (SegmentSelector, SegmentSelector) {
        let mut cpus = CPUS.lock();
        (cpus.current_cpu().selectors.user_code, cpus.current_cpu().selectors.user_data)
    }
}

pub fn build_gdt(tss: &'static TaskStateSegment) -> (GlobalDescriptorTable, Selectors) {
    let mut gdt = GlobalDescriptorTable::new();

    let kernel_code = Descriptor::kernel_code_segment();
    let kernel_data = Descriptor::kernel_data_segment();
    let user_code = Descriptor::user_code_segment();
    let user_data = Descriptor::user_data_segment();

    // The order is required.
    let kernel_code_selector = gdt.append(kernel_code);
    let kernel_data_selector = gdt.append(kernel_data);

    let user_data_selector = gdt.append(user_data);
    let user_code_selector = gdt.append(user_code);

    let tss_selector = gdt.append(Descriptor::tss_segment(tss));

    let selectors = Selectors {
        kernel_code: kernel_code_selector,
        kernel_data: kernel_data_selector,
        user_code: user_code_selector,
        user_data: user_data_selector,
        tss: tss_selector,
    };

    (gdt, selectors)
}

pub fn load_gdt(gdt: &'static GlobalDescriptorTable, selectors: &Selectors) {
    gdt.load();

    unsafe {
        use x86_64::instructions::segmentation::Segment;

        CS::set_reg(selectors.kernel_code);
        DS::set_reg(selectors.kernel_data);
        ES::set_reg(selectors.kernel_data);
        FS::set_reg(selectors.kernel_data);
        GS::set_reg(selectors.kernel_data);
        SS::set_reg(selectors.kernel_data);

        load_tss(selectors.tss);
    }
}

/*pub static BP_TSS: Lazy<Mutex<TaskStateSegment>> = Lazy::new(|| {
    Mutex::new(tss!({
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
        unsafe { addr_of!(STACK) }
    }))
});
pub static BP_GDT: Lazy<(GlobalDescriptorTable, Selectors)> = Lazy::new(|| {
    build_gdt(unsafe {
        let tss_ptr: *const TaskStateSegment = &*BP_TSS.lock();
        &*tss_ptr
    })
});*/

pub fn init() {
    //load(&BP_GDT.0, &BP_GDT.1);
    //.get_mut(&BSP_ID).unwrap().init();

    log::info!("[GDT] initialized");
}

/*pub struct ApInfo {
    gdt: &'static GlobalDescriptorTable,
    pub tss: &'static mut TaskStateSegment,
    selectors: Selectors,
}

pub fn allocate_for_ap() -> ApInfo {
    let tss = Box::leak(Box::new(tss!(&vec![0u8; STACK_SIZE].leak())));
    let (gdt, selectors) = build(tss);
    let gdt = Box::leak(Box::new(gdt));
    ApInfo { gdt, tss: crate::ref_to_mut(tss), selectors }
}

pub fn init_ap(info: &ApInfo) {
    load(info.gdt, &info.selectors);
}*/
