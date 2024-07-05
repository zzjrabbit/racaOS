use alloc::vec::Vec;
use conquer_once::spin::OnceCell;
use limine::{request::SmpRequest, smp::Cpu};
use spin::Mutex;

use crate::{arch::interrupts::IDT, task::scheduler::SCHEDULER};

#[used]
#[link_section = ".requests"]
static SMP_REQUEST: SmpRequest = SmpRequest::new();

pub static AP_INFO: OnceCell<Mutex<Vec<Option<super::gdt::ApInfo>>>> = OnceCell::uninit();

pub fn init() {
    let smp_response = SMP_REQUEST.get_response().unwrap();

    let mut infos = Vec::new();
    for _ in 0..smp_response.cpus().len() {
        infos.push(Some(super::gdt::allocate_for_ap()));
    }

    AP_INFO.init_once(|| Mutex::new(infos));

    for cpu in smp_response.cpus() {
        if cpu.id == smp_response.bsp_lapic_id() {
            continue;
        }
        cpu.goto_address.write(ap_entry);
    }
}

extern "C" fn ap_entry(smp_info: &Cpu) -> ! {
    crate::println!("Processor:{}",smp_info.id);

    let info = AP_INFO.try_get().unwrap().lock();

    super::gdt::init_ap(info[smp_info.id as usize].as_ref().unwrap());
    drop(info);

    IDT.load();
    super::apic::init_ap();

    while !SCHEDULER.is_initialized() {}

    x86_64::instructions::interrupts::enable();

    loop {
        x86_64::instructions::hlt();
    }
}
