use core::sync::atomic::Ordering;

use alloc::{collections::BTreeMap, vec::Vec};
use conquer_once::spin::{Lazy, OnceCell};
use cpu::{CpuManager, CPU};
use limine::{request::SmpRequest, smp::Cpu};
use spin::Mutex;

use crate::task::scheduler::SCHEDULER_INIT;

use super::apic::get_lapic_addr;

mod cpu;

#[used]
#[link_section = ".requests"]
static SMP_REQUEST: SmpRequest = SmpRequest::new();

//pub static AP_INFO: OnceCell<Mutex<Vec<Option<super::gdt::ApInfo>>>> = OnceCell::uninit();
pub static CPUS: Lazy<Mutex<CpuManager>> =
    Lazy::new(|| Mutex::new(CpuManager::new(SMP_REQUEST.get_response().unwrap())));
pub static BSP_ID: Lazy<u32> = Lazy::new(|| {
    let smp_response = SMP_REQUEST.get_response().unwrap();

    smp_response.bsp_lapic_id()
});

pub fn ap_idle() {
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn init() {
    let smp_response = SMP_REQUEST.get_response().unwrap();

    /*let mut infos = Vec::new();
    for _ in 0..smp_response.cpus().len() {
        infos.push(Some(super::gdt::allocate_for_ap()));
    }

    AP_INFO.init_once(|| Mutex::new(infos));*/

    for _ in 0..smp_response.cpus().len() {
        crate::task::Thread::new_kernel_thread(ap_idle);
    }

    for cpu in smp_response.cpus() {
        if cpu.id == smp_response.bsp_lapic_id() {
            continue;
        }
        cpu.goto_address.write(ap_entry);
    }
}

extern "C" fn ap_entry(smp_info: &Cpu) -> ! {
    crate::println!("Processor:{}", smp_info.id);

    /*let info = AP_INFO.try_get().unwrap().lock();

    super::gdt::init_ap(info[smp_info.id as usize].as_ref().unwrap());
    drop(info);

    super::interrupts::init();
    super::apic::init_ap();

    crate::user::init();*/

    let mut cpus = CPUS.lock();
    let cpu = cpus.get_mut(&smp_info.lapic_id).unwrap();
    cpu.init();
    drop(cpus);

    while !SCHEDULER_INIT.load(Ordering::Relaxed) {}

    //x86_64::instructions::interrupts::enable();

    loop {
        x86_64::instructions::hlt();
    }
}
