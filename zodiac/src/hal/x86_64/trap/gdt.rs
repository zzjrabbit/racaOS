use core::ptr::addr_of;

use spin::Lazy;
use x86_64::VirtAddr;
use x86_64::instructions::segmentation::{CS, SS, Segment};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::GlobalDescriptorTable;
use x86_64::structures::gdt::{Descriptor, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;

pub struct CpuInfo {
    gdt: GlobalDescriptorTable,
    tss: TaskStateSegment,
    selectors: Option<Selectors>,
}

impl Default for CpuInfo {
    fn default() -> Self {
        Self {
            gdt: GlobalDescriptorTable::new(),
            tss: TaskStateSegment::new(),
            selectors: None,
        }
    }
}

impl CpuInfo {
    pub fn tss_mut(&mut self) -> &mut TaskStateSegment {
        &mut self.tss
    }

    #[inline]
    pub fn set_ring0_rsp(&mut self, rsp: VirtAddr) {
        self.tss.privilege_stack_table[0] = rsp;
    }

    #[inline]
    pub fn get_ring0_rsp(&self) -> VirtAddr {
        self.tss.privilege_stack_table[0]
    }

    #[inline]
    pub fn set_ring3_rsp(&mut self, rsp: VirtAddr) {
        self.tss.privilege_stack_table[1] = rsp;
    }

    #[inline]
    pub fn get_ring3_rsp(&self) -> VirtAddr {
        self.tss.privilege_stack_table[1]
    }
}

impl CpuInfo {
    pub fn init(&mut self) {
        let (mut gdt, mut selectors) = COMMON_GDT.clone();

        let tss_ref = unsafe { &*addr_of!(self.tss) };
        let tss_selector = Some(gdt.append(Descriptor::tss_segment(tss_ref)));
        selectors.tss_selector = tss_selector;

        self.gdt = gdt;
        self.selectors = Some(selectors);

        let gdt_ptr: *const _ = &self.gdt;
        unsafe { (*gdt_ptr).load() }

        let selectors = &self.selectors.as_ref().unwrap();
        unsafe {
            CS::set_reg(selectors.code_selector);
            SS::set_reg(selectors.data_selector);
            load_tss(selectors.tss_selector.unwrap());
        }
    }

    pub fn kernel_code_selector(&self) -> usize {
        self.selectors.as_ref().unwrap().code_selector.0 as usize
    }

    pub fn kernel_data_selector(&self) -> usize {
        self.selectors.as_ref().unwrap().data_selector.0 as usize
    }

    pub fn user_code_selector(&self) -> usize {
        self.selectors.as_ref().unwrap().user_code_selector.0 as usize
    }

    pub fn user_data_selector(&self) -> usize {
        self.selectors.as_ref().unwrap().user_data_selector.0 as usize
    }
}

static COMMON_GDT: Lazy<(GlobalDescriptorTable, Selectors)> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();

    let code_selector = gdt.append(Descriptor::kernel_code_segment());
    let data_selector = gdt.append(Descriptor::kernel_data_segment());
    let user_data_selector = gdt.append(Descriptor::user_data_segment());
    let user_code_selector = gdt.append(Descriptor::user_code_segment());

    //log::info!("kernel code selector: {:x?}, kernel data selector: {:x?}", code_selector.0, data_selector.0);
    //log::info!("user code selector: {:x?}, user data selector: {:x?}", user_code_selector.0, user_data_selector.0);

    let selectors = Selectors {
        code_selector,
        data_selector,
        user_data_selector,
        user_code_selector,
        tss_selector: None,
    };

    (gdt, selectors)
});

#[derive(Clone)]
pub struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    user_code_selector: SegmentSelector,
    user_data_selector: SegmentSelector,
    tss_selector: Option<SegmentSelector>,
}
