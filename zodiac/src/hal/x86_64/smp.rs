use alloc::collections::BTreeMap;
use limine::request::MpRequest;
use spin::{Lazy, RwLock};

use crate::hal::cpu::Cpu;

use super::ap_entry;
use super::trap::gdt::CpuInfo;

#[used]
#[unsafe(link_section = ".requests")]
pub(super) static MP_REQUEST: MpRequest = MpRequest::new();

pub static BSP_LAPIC_ID: Lazy<u32> =
    Lazy::new(|| MP_REQUEST.get_response().unwrap().bsp_lapic_id());

pub static CPUS: Lazy<Cpus> = Lazy::new(Cpus::default);

pub struct Cpus(RwLock<BTreeMap<Cpu, CpuInfo>>);

impl Default for Cpus {
    fn default() -> Self {
        let mut cpus = BTreeMap::new();
        cpus.insert(Cpu::bsp(), CpuInfo::default());
        Cpus(RwLock::new(cpus))
    }
}

impl Cpus {
    pub fn len(&self) -> usize {
        self.0.read().len()
    }
}

impl Cpus {
    pub fn load(&self, cpu: Cpu) {
        let mut inner = self.0.write();
        let cpu_info = inner.get_mut(&cpu).unwrap();
        cpu_info.init();
    }

    pub fn add_cpu(&self, cpu: Cpu) {
        let cpu_info = CpuInfo::default();
        self.0.write().insert(cpu, cpu_info);
    }

    pub fn init_ap(&self) {
        let response = MP_REQUEST.get_response().unwrap();

        for cpu in response.cpus() {
            if cpu.lapic_id != *BSP_LAPIC_ID {
                self.add_cpu(Cpu::new(cpu.lapic_id));
                cpu.goto_address.write(ap_entry);
            }
        }
    }

    pub fn with_cpu_info<F, R>(&self, cpu: Cpu, f: F) -> R
    where
        F: FnOnce(&CpuInfo) -> R,
    {
        let inner = self.0.read();
        let cpu_info = inner.get(&cpu).unwrap();
        f(cpu_info)
    }

    pub fn with_cpu_info_mut<F, R>(&self, cpu: Cpu, f: F) -> R
    where
        F: FnOnce(&mut CpuInfo) -> R,
    {
        let mut inner = self.0.write();
        let cpu_info = inner.get_mut(&cpu).unwrap();
        f(cpu_info)
    }
}
