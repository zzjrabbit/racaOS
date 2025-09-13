use bit_field::BitField;
use core::ptr;
use core::time::Duration;
use spin::Lazy;

use crate::hal::context::TrapFrame;
use crate::hal::kernel::ACPI;
use crate::mem::{
    MMUFlags, PageSize, PhysicalAddress, PhysicalMemoryAllocOptions, VirtualMemorySpace,
    convert_physical_to_virtual,
};
use crate::timer::TIMER_CALLBACKS;
use crate::trap::Irq;

pub fn init() {
    Lazy::force(&HPET);
}

pub static HPET: Lazy<Hpet> = Lazy::new(|| {
    let origin_physical_address = ACPI.hpet_info.base_address as PhysicalAddress;
    let physical_address = PageSize::Size4K.align_down(origin_physical_address);
    let virtual_address = convert_physical_to_virtual(physical_address);

    let page_count = PageSize::Size4K.align_up(origin_physical_address + 0x1000 - physical_address)
        / PageSize::Size4K as usize;

    let pm = PhysicalMemoryAllocOptions::default()
        .count(page_count)
        .contiguous(true)
        .address(physical_address)
        .allocate()
        .unwrap();

    let vm_space = VirtualMemorySpace::new_kernel();
    let _ = vm_space
        .cursor(virtual_address, PageSize::Size4K)
        .unwrap()
        .map(&pm, MMUFlags::READ | MMUFlags::WRITE);

    Hpet::new(virtual_address as u64)
});

pub struct Hpet {
    address: u64,
    fms_per_tick: u64,
}

impl Hpet {
    pub fn ticks(&self) -> u64 {
        let counter_addr = (self.address + 0xf0) as *const u64;
        unsafe { ptr::read_volatile(counter_addr) }
    }

    pub fn elapsed(&self) -> Duration {
        let ticks = self.ticks();
        Duration::from_nanos(ticks * self.fms_per_tick / 1_000_000)
    }

    pub fn estimate(&self, duration: Duration) -> u64 {
        let ticks = self.ticks();
        ticks + (duration.as_nanos() as u64 * 1_000_000 / self.fms_per_tick)
    }

    pub fn set_timer(&self, value: u64) {
        let comparator_addr = (self.address + 0x108) as *mut u64;
        unsafe { ptr::write_volatile(comparator_addr, value) };
    }
}

impl Hpet {
    pub fn new(address: u64) -> Self {
        let general_ptr = address as *const u64;
        let general_info = unsafe { ptr::read_volatile(general_ptr) };

        let fms_per_tick = general_info.get_bits(32..64);
        let counter_addr = (address + 0xf0) as *const u64;
        unsafe { ptr::write_volatile(counter_addr as *mut u64, 0) };

        let hpet = Self {
            address,
            fms_per_tick,
        };

        unsafe {
            let enable_cnf_addr = (hpet.address + 0x10) as *mut u64;
            let old_cnf = ptr::read_volatile(enable_cnf_addr);
            ptr::write_volatile(enable_cnf_addr, old_cnf | 1);

            let timer_config_addr = (hpet.address + 0x100) as *mut u64;
            let old_config = ptr::read_volatile(timer_config_addr);
            let route_cap = old_config.get_bits(32..63);

            let mut irq = None;
            for i in 0..32 {
                if route_cap.get_bit(i) {
                    irq = Irq::allocate_specific(i as u8, hpet_timer_handler);
                }
                if irq.is_some() {
                    let timer_config = ((i as u64) << 9) | (1 << 2);
                    ptr::write_volatile(timer_config_addr, timer_config);
                    return hpet;
                }
            }

            panic!("Unable to allocate IRQ for HPET timer");
        }
    }
}

fn hpet_timer_handler(frame: &mut TrapFrame) {
    for callback in TIMER_CALLBACKS.read().iter() {
        callback(frame);
    }
}
