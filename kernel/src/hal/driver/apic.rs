use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use derive_more::Deref;
use spin::{Lazy, Mutex};
use x2apic::ioapic::{IoApic, IrqMode, RedirectionTableEntry};
use x2apic::lapic::{LocalApic, LocalApicBuilder, TimerMode};
use x86_64::{PhysAddr, instructions::port::Port};

use super::super::mem::convert_physical_to_virtual;
use super::acpi::ACPI;
use super::hpet::HPET;
use crate::hal::int::IntFrame;

const TIMER_FREQUENCY_HZ: u32 = 250;
const TIMER_CALIBRATION_ITERATION: u32 = 100;
const IOAPIC_INTERRUPT_INDEX_OFFSET: u8 = 32;

pub static APIC_INIT: AtomicBool = AtomicBool::new(false);
pub static CALIBRATED_TIMER_INITIAL: AtomicU32 = AtomicU32::new(0);

fn timer_handler(_frame: &mut IntFrame) {}

fn apic_error_handler(_frame: &mut IntFrame) {
    panic!("Apic Error");
}

fn spurious_handler(_frame: &mut IntFrame) {
    log::info!("Spurious Interrupt");
}

#[derive(Deref)]
pub struct LockedLocalApic(Mutex<LocalApic>);

unsafe impl Send for LockedLocalApic {}
unsafe impl Sync for LockedLocalApic {}

pub static LAPIC: Lazy<LockedLocalApic> = Lazy::new(|| unsafe {
    let physical_address = PhysAddr::new(ACPI.apic.local_apic_address);
    let virtual_address = convert_physical_to_virtual(physical_address);

    let timer_int = crate::hal::int::register_handler(timer_handler).unwrap();
    let apic_error_int = crate::hal::int::register_handler(apic_error_handler).unwrap();
    let spurious_int = crate::hal::int::register_handler(spurious_handler).unwrap();

    let mut lapic = LocalApicBuilder::new()
        .timer_vector(timer_int)
        .timer_mode(TimerMode::OneShot)
        .timer_initial(0)
        .error_vector(apic_error_int)
        .spurious_vector(spurious_int)
        .set_xapic_base(virtual_address.as_u64())
        .build()
        .unwrap_or_else(|err| panic!("Failed to build local APIC: {:#?}", err));

    lapic.enable();

    LockedLocalApic(Mutex::new(lapic))
});

pub static IOAPIC: Lazy<Mutex<IoApic>> = Lazy::new(|| unsafe {
    let physical_address = PhysAddr::new(ACPI.apic.io_apics[0].address as u64);
    let virtual_address = convert_physical_to_virtual(physical_address);

    let mut ioapic = IoApic::new(virtual_address.as_u64());
    ioapic.init(IOAPIC_INTERRUPT_INDEX_OFFSET);
    Mutex::new(ioapic)
});

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum IrqVector {
    Keyboard = 1,
    Mouse = 12,
    HpetTimer,
}

pub fn init() {
    unsafe {
        disable_pic();
        calibrate_timer();
    };

    APIC_INIT.store(true, Ordering::SeqCst);
    log::info!("APIC initialized successfully!");
}

#[inline]
pub fn end_of_interrupt() {
    unsafe {
        LAPIC.lock().end_of_interrupt();
    }
}

unsafe fn disable_pic() {
    unsafe {
        Port::<u8>::new(0x21).write(0xff);
        Port::<u8>::new(0xa1).write(0xff);
    }
}

/// # Safety
/// This function is quite safe
pub unsafe fn ioapic_add_entry(irq: u8, vector: u8) {
    let lapic = LAPIC.lock();
    let mut ioapic = IOAPIC.lock();
    let mut entry = RedirectionTableEntry::default();
    entry.set_mode(IrqMode::Fixed);
    entry.set_dest(unsafe { lapic.id() } as u8);
    entry.set_vector(vector);
    unsafe {
        ioapic.set_table_entry(irq, entry);
        ioapic.enable_irq(irq);
    }
}

/// # Safety
/// This function is quite safe
pub unsafe fn calibrate_timer() {
    let mut lapic = LAPIC.lock();
    let mut lapic_total_ticks = 0;

    for _ in 0..TIMER_CALIBRATION_ITERATION {
        let last_time = HPET.elapsed().as_nanos();
        unsafe {
            lapic.set_timer_initial(!0);
        }
        while HPET.elapsed().as_nanos() - last_time < 1_000_000 {}
        lapic_total_ticks += !0 - unsafe { lapic.timer_current() };
    }

    let average_clock_per_ms = lapic_total_ticks / TIMER_CALIBRATION_ITERATION;
    let calibrated_timer_initial = average_clock_per_ms * 1000 / TIMER_FREQUENCY_HZ;
    log::debug!("Calibrated timer initial: {}", calibrated_timer_initial);

    unsafe {
        lapic.set_timer_mode(TimerMode::Periodic);
        lapic.set_timer_initial(calibrated_timer_initial);
    }
    CALIBRATED_TIMER_INITIAL.store(calibrated_timer_initial, Ordering::SeqCst);
}
