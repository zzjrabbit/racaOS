/*
* @file    :   resource.rs
* @time    :   2023/12/16 13:43:51
* @author  :   zzjcarrot
*/

//use core::alloc::Layout;

use core::ops::{Deref, DerefMut};

//use alloc::alloc::alloc;
use alloc::collections::BTreeMap;
use x86_64::instructions::tables::load_tss;
use x86_64::registers::segmentation::*;
use x86_64::{
    structures::{gdt::GlobalDescriptorTable, tss::TaskStateSegment},
    VirtAddr,
};

use crate::arch::apic::{get_lapic, get_lapic_id};
use crate::arch::interrupts::IDT;
use crate::arch::smp::ap_entry;
use crate::user;

use super::super::apic::calibrate_timer;
use super::super::gdt::Selectors;
use super::super::gdt::DOUBLE_FAULT_IST_INDEX;
use super::{BSP_ID, SMP_REQUEST};

// 处理器结构体
pub struct CPU {
    pub gdt: GlobalDescriptorTable,
    pub tss: TaskStateSegment,
    pub selectors: Option<Selectors>,
    pub double_fault_stack: [u8;128],
}

impl CPU {
    pub fn new() -> Self {
        Self {
            gdt: GlobalDescriptorTable::new(),
            tss: TaskStateSegment::new(),
            selectors: None,
            double_fault_stack: [0; 128],
        }
    }

    pub fn init(&mut self) {
        let stack_start = self.double_fault_stack.as_ptr() as u64;
        let stack_end = VirtAddr::new(stack_start + self.double_fault_stack.len() as u64);

        self.tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = stack_end;
        self.selectors = Some(Selectors::new(&mut self.gdt, &self.tss));
    }

    pub fn load(&self) {
        let gdt_ptr: *const _ = &self.gdt;
        unsafe { (&*gdt_ptr).load() }

        let selectors = &self.selectors.as_ref().unwrap();
        unsafe {
            CS::set_reg(selectors.code_selector);
            SS::set_reg(selectors.data_selector);
            load_tss(selectors.tss_selector);
        }

        IDT.load();
    }

    pub fn init_last(&self) {
        let mut lapic = get_lapic();
        log::info!("OK");
        unsafe {

            lapic.enable();

            calibrate_timer(&mut lapic);

        }

        user::init();
    }

    pub fn set_ring0_rsp(&mut self, rsp: VirtAddr) {
        self.tss.privilege_stack_table[0] = rsp;
    }
}

pub struct CpuManager {
    bsp: CPU,
    cpus: BTreeMap<u32, CPU>,
}

impl CpuManager {
    pub fn new() -> CpuManager {
        CpuManager { cpus: BTreeMap::new(),bsp: CPU::new() }
    }

    pub fn init_bsp(&mut self) {

        self.bsp.init();
        self.bsp.load();

        let tss_ptr = &self.bsp.tss as *const _;
        log::warn!("bsp tss_ptr: {:#x}", tss_ptr as u64);

        let stack_start = self.bsp.double_fault_stack.as_ptr();
        log::warn!("bsp stack start: {:#x}", stack_start as u64);
    }

    pub fn init_ap(&mut self) {
        let response = SMP_REQUEST.get_response().unwrap();

        for cpu in response.cpus() {
            if cpu.id != *BSP_ID {
                let info = CPU::new();
                self.cpus.insert(cpu.lapic_id, info);

                let info = self.cpus.get_mut(&cpu.lapic_id).unwrap();
                info.init();

                cpu.goto_address.write(ap_entry);
                log::info!("AP CPU {} initialized!", cpu.lapic_id);

                let tss_ptr = &info.tss as *const _;
                log::warn!("ap tss_ptr: {:#x}", tss_ptr as u64);

                let stack = &self.cpus.get(&cpu.id).unwrap().double_fault_stack;
                let stack_end = stack.as_ptr() as u64 + stack.len() as u64;
                log::warn!("ap stack start: {:#x}", stack_end as u64);
            }
        }
    }

    pub fn get_cpu(&mut self, id: usize) -> &mut CPU {
        if id == *BSP_ID as usize {
            self.bsp_cpu()
        } else {
            self.cpus
                .get_mut(&(id as u32))
                .unwrap_or_else(|| panic!("CPU {} not found!", id))
        }
    }

    pub fn bsp_cpu(&mut self) -> &mut CPU {
        &mut self.bsp
    }

    pub fn current_cpu(&mut self) -> (u32, &mut CPU) {
        let current_cpu_id = get_lapic_id();
        (current_cpu_id, self.get_cpu(current_cpu_id as usize))
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
