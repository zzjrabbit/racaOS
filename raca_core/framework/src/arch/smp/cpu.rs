/*
* @file    :   resource.rs
* @time    :   2023/12/16 13:43:51
* @author  :   zzjcarrot
*/

//use core::alloc::Layout;

use core::ops::{Deref, DerefMut};

//use alloc::alloc::alloc;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use limine::response::SmpResponse;
use x2apic::lapic::LocalApic;
use x86_64::instructions::tables::load_tss;
use x86_64::registers::segmentation::*;
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::{
    structures::{
        gdt::GlobalDescriptorTable,
        tss::TaskStateSegment,
    },
    VirtAddr,
};

use crate::arch::apic::{get_lapic, get_lapic_id};
use crate::arch::gdt::{build_gdt, load_gdt};
use crate::{println, ref_to_mut, ref_to_static, user};

use super::super::apic::calibrate_timer;
use super::super::gdt::Selectors;
use super::super::gdt::DOUBLE_FAULT_IST_INDEX;
use super::super::interrupts::*;
use super::BSP_ID;

// 处理器结构体
pub struct CPU {
    pub gdt: GlobalDescriptorTable,
    pub tss: &'static mut TaskStateSegment,
    pub selectors: Selectors,
    pub idt: InterruptDescriptorTable,
    pub lapic: Box<LocalApic>,
}

impl CPU {
    pub fn new() -> Self {
        //let tss_size = core::mem::size_of::<TaskStateSegment>();
        //let gdt_size = core::mem::size_of::<GlobalDescriptorTable>();
        //let idt_size = core::mem::size_of::<InterruptDescriptorTable>();
        let tss = Box::new(TaskStateSegment::new());

        ref_to_mut(tss.as_ref()).interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { core::ptr::addr_of!(STACK) });
            stack_start + STACK_SIZE as u64
        };

        let tss = Box::leak(tss);

        let (gdt,selectors) = build_gdt(tss);

        let idt = build_idt();


        //let lapic_size = core::mem::size_of::<LocalApic>();
        let lapic = Box::new(get_lapic());

        Self {
            gdt,
            tss: ref_to_mut(tss),
            selectors,
            idt,
            lapic,
        }
    }

    pub fn init(&mut self) {

        let selectors = &self.selectors;
        load_gdt(ref_to_static(&self.gdt), selectors);

        let idt_ptr: *const _ = &self.idt;

        unsafe { (&*idt_ptr).load() }

        x86_64::instructions::interrupts::disable();
        unsafe {
            self.lapic.enable();
            calibrate_timer(self.lapic.as_mut());
            //loop {}
            if !self.lapic.is_bsp() {
                self.lapic.enable_timer();
            }
        }
        log::info!("CPU {} init", self.id());
        x86_64::instructions::interrupts::disable();
        user::init();
        log::info!("CPU {} init", self.id());
    }

    pub fn set_ring0_rsp(&mut self, rsp: VirtAddr) {
        self.tss.privilege_stack_table[0] = rsp;
    }

    pub fn id(&self) -> u32 {
        unsafe { self.lapic.id() }
    }
}

pub struct CpuManager {
    cpus: BTreeMap<u32, CPU>,
}

impl CpuManager {
    pub fn new(sm_response: &SmpResponse) -> CpuManager {
        let mut cpus = BTreeMap::new();
        for cpu in sm_response.cpus().iter() {
            cpus.insert(cpu.lapic_id, CPU::new());
        }
        CpuManager { cpus }
    }

    pub fn current_cpu(&mut self) -> &mut CPU {
        self.cpus.get_mut(&get_lapic_id()).unwrap()
    }

    pub fn bsp_cpu(&mut self) -> &mut CPU {
        self.cpus.get_mut(&BSP_ID).unwrap()
    }
}

impl Deref for CpuManager {
    type Target = BTreeMap<u32, CPU>;

    fn deref(&self) -> &Self::Target {
        &self.cpus
    }
}

impl DerefMut for CpuManager {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cpus
    }
}
