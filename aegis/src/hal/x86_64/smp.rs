use alloc::collections::BTreeMap;
use limine::request::MpRequest;
use spin::{Lazy, RwLock};

use super::ap_entry;
use super::trap::gdt::CpuInfo;

#[used]
#[unsafe(link_section = ".requests")]
static MP_REQUEST: MpRequest = MpRequest::new();

pub static BSP_LAPIC_ID: Lazy<u32> =
    Lazy::new(|| MP_REQUEST.get_response().unwrap().bsp_lapic_id());

pub static CPUS: Lazy<Cpus> = Lazy::new(|| Cpus::default());

pub struct Cpus(RwLock<BTreeMap<u32, CpuInfo>>);

impl Default for Cpus {
    fn default() -> Self {
        let mut cpus = BTreeMap::new();
        cpus.insert(*BSP_LAPIC_ID, CpuInfo::default());
        Cpus(RwLock::new(cpus))
    }
}

impl Cpus {
    pub fn load(&self, lapic_id: u32) {
        let mut inner = self.0.write();
        let cpu_info = inner.get_mut(&lapic_id).unwrap();
        cpu_info.init();
    }
    
    pub fn add_cpu(&self, lapic_id: u32) {
        self.0.write().insert(lapic_id, CpuInfo::default());
    }

    pub fn init_ap(&self) {
        let response = MP_REQUEST.get_response().unwrap();
        
        for cpu in response.cpus() {
            if cpu.lapic_id != *BSP_LAPIC_ID {
                self.add_cpu(cpu.lapic_id);
                cpu.goto_address.write(ap_entry);
            }
        }
    }
}
