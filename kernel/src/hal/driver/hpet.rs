use alloc::sync::Arc;
use bit_field::BitField;
use core::ptr;
use core::time::Duration;
use spin::{Lazy, mutex::Mutex};
use x86_64::PhysAddr;

use super::super::mem::convert_physical_to_virtual;
use super::acpi::ACPI;
use super::apic::IrqVector;
use crate::{
    hal::ref_current_page_table,
    mm::{MMUFlags, PhysicalMemory, VirtualMemory, VmMapping},
};

pub static HPET: Lazy<Hpet> = Lazy::new(|| {
    let physical_address = PhysAddr::new(ACPI.hpet_info.base_address as u64);
    let virtual_address = convert_physical_to_virtual(physical_address);

    let virtual_memory = VirtualMemory::new(
        virtual_address.as_u64() as usize,
        1,
        Arc::new(Mutex::new(ref_current_page_table())),
    );
    let physical_memory = PhysicalMemory::new(physical_address.as_u64() as usize, 1);
    let mapping = Arc::new(VmMapping::new(
        MMUFlags::WRITE | MMUFlags::READ,
        virtual_memory.clone(),
        physical_memory.clone(),
    ));
    let _ = mapping.map();

    Hpet::new(virtual_address.as_u64())
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

            // 这里只是给个警告，一般是支持的（我在真机和QEMU上试过了）
            // 最好是根据它支持的vector来动态设置IrqVector::HpetTimer
            // 但是enum没法改嘛，只能硬编码了
            //
            // 这个cap的意思是，哪一位是1就表明那个Vector是支持的
            if !route_cap.get_bit(IrqVector::HpetTimer as usize) {
                log::warn!("HPET timer does not support our IRQ vector!");
                log::info!("Timer route capabilities: {:#032b}", route_cap);
            }

            let timer_config = ((IrqVector::HpetTimer as u64) << 9) | (1 << 2);
            ptr::write_volatile(timer_config_addr, timer_config);
        }

        hpet
    }
}
