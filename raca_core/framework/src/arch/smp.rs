use core::sync::atomic::Ordering;

use alloc::collections::BTreeMap;
use limine::request::SmpRequest;
use limine::smp::Cpu;
use spin::{Lazy, Mutex};

use super::apic::{calibrate_timer, get_lapic_id};
use super::gdt::CpuInfo;
use super::interrupts::IDT;
use crate::arch::apic::get_lapic;
use crate::drivers::hpet::HPET_INIT;
use crate::task::scheduler::{Scheduler, SCHEDULERS, SCHEDULER_INIT};
use crate::{user, START_SCHEDULE};

#[used]
#[link_section = ".requests"]
static SMP_REQUEST: SmpRequest = SmpRequest::new();

pub static CPUS: Lazy<Mutex<Cpus>> = Lazy::new(|| Mutex::new(Cpus::new()));

unsafe extern "C" fn ap_entry(smp_info: &Cpu) -> ! {
    CPUS.lock().get_cpu(smp_info.lapic_id as usize).load();
    IDT.load();

    while !HPET_INIT.load(Ordering::Relaxed) {}

    let mut lapic = get_lapic();
    lapic.enable();
    calibrate_timer(&mut lapic);
    lapic.enable_timer();

    while !SCHEDULER_INIT.load(Ordering::Relaxed) {}
    SCHEDULERS
        .lock()
        .insert(smp_info.lapic_id, Scheduler::new());

    user::init();

    while !START_SCHEDULE.load(Ordering::Relaxed) {}
    x86_64::instructions::interrupts::enable();

    loop {
        x86_64::instructions::hlt();
    }
}

pub struct Cpus {
    bsp: CpuInfo,
    bsp_lapic_id: u32,
    ap_infos: BTreeMap<u32, CpuInfo>,
}

impl Cpus {
    pub fn new() -> Self {
        let response = SMP_REQUEST.get_response().unwrap();

        Self {
            bsp: CpuInfo::new(),
            bsp_lapic_id: response.bsp_lapic_id(),
            ap_infos: BTreeMap::new(),
        }
    }

    pub fn init_bsp(&mut self) {
        self.bsp.init();
        self.bsp.load();
    }

    pub fn init_ap(&mut self) {
        let response = SMP_REQUEST.get_response().unwrap();

        for cpu in response.cpus() {
            if cpu.lapic_id != self.bsp_lapic_id {
                let info = CpuInfo::new();
                self.ap_infos.insert(cpu.lapic_id, info);

                let info = self.ap_infos.get_mut(&cpu.lapic_id).unwrap();
                info.init();

                cpu.goto_address.write(ap_entry);
            }
        }
    }

    pub fn get_cpu(&mut self, id: usize) -> &mut CpuInfo {
        if id == self.bsp_lapic_id as usize {
            self.bsp_cpu()
        } else {
            self.ap_infos
                .get_mut(&(id as u32))
                .unwrap_or_else(|| panic!("CPU {} not found!", id))
        }
    }

    pub fn bsp_cpu(&mut self) -> &mut CpuInfo {
        &mut self.bsp
    }

    pub fn current_cpu(&mut self) -> (u32, &mut CpuInfo) {
        let current_cpu_id = get_lapic_id();
        (current_cpu_id, self.get_cpu(current_cpu_id as usize))
    }

    pub fn bsp_id(&self) -> u32 {
        self.bsp_lapic_id
    }

    pub fn cpu_num(&self) -> usize {
        self.ap_infos.len() + 1
    }
}
