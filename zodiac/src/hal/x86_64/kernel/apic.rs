use core::sync::atomic::{AtomicBool, Ordering};
use x86_64::instructions::port::Port;

use derive_more::Deref;
use spin::{Lazy, Mutex};
use x2apic::lapic::{LocalApic, LocalApicBuilder, TimerMode};

use super::acpi::ACPI;
use crate::{
    hal::context::TrapFrame,
    mem::{
        MMUFlags, PageSize, PhysicalAddress, PhysicalMemoryAllocOptions, VirtualMemorySpace,
        convert_physical_to_virtual,
    },
    task::schedule,
    trap::Irq,
};

static APIC_INIT: AtomicBool = AtomicBool::new(false);

#[derive(Deref)]
pub struct LockedLocalApic(Mutex<LocalApic>);

unsafe impl Send for LockedLocalApic {}
unsafe impl Sync for LockedLocalApic {}

fn timer_handler(frame: &mut TrapFrame) {
    schedule(frame);
}

static TIMER_IRQ: Lazy<Irq> = Lazy::new(|| Irq::allocate(timer_handler).unwrap());
static APIC_ERROR_IRQ: Lazy<Irq> = Lazy::new(|| Irq::allocate(|_frame| {}).unwrap());
static SPURIOUS_IRQ: Lazy<Irq> = Lazy::new(|| Irq::allocate(|_frame| {}).unwrap());

pub static LAPIC: Lazy<LockedLocalApic> = Lazy::new(|| unsafe {
    let origin_physical_address = ACPI.apic.local_apic_address as PhysicalAddress;
    let physical_address = PageSize::Size4K.align_down(origin_physical_address);
    log::trace!(
        "Initializing LAPIC at physical address {:#x}, timer vector: {}, spurious vector: {}, error vector: {}",
        physical_address,
        TIMER_IRQ.as_int_vector(),
        SPURIOUS_IRQ.as_int_vector(),
        APIC_ERROR_IRQ.as_int_vector()
    );
    let virtual_address = convert_physical_to_virtual(physical_address);

    let page_count = PageSize::Size4K.align_up(origin_physical_address + 0x1000 - physical_address)
        / PageSize::Size4K as usize;

    let physical_memory = PhysicalMemoryAllocOptions::default()
        .contiguous(true)
        .address(physical_address)
        .count(page_count)
        .allocate()
        .unwrap();
    let vm_space = VirtualMemorySpace::new_kernel();

    vm_space
        .cursor(virtual_address, PageSize::Size4K)
        .unwrap()
        .map(&physical_memory, MMUFlags::READ | MMUFlags::WRITE)
        .unwrap();

    let mut lapic = LocalApicBuilder::new()
        .timer_vector(TIMER_IRQ.as_int_vector() as usize)
        .timer_mode(TimerMode::OneShot)
        .timer_initial(0)
        .error_vector(APIC_ERROR_IRQ.as_int_vector() as usize)
        .spurious_vector(SPURIOUS_IRQ.as_int_vector() as usize)
        .set_xapic_base(virtual_address as u64)
        .build()
        .unwrap_or_else(|err| panic!("Failed to build local APIC: {:#?}", err));

    lapic.enable();
    lapic.disable_timer();
    
    log::trace!("Lapic Initialized");

    LockedLocalApic(Mutex::new(lapic))
});

pub fn apic_timer_irq() -> Irq {
    TIMER_IRQ.clone()
}

pub fn ap_init() {
    while !APIC_INIT.load(Ordering::SeqCst) {
        core::hint::spin_loop();
    }
    unsafe {
        LAPIC.lock().enable();
        LAPIC.lock().disable_timer();
    }
}

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

pub fn init() {
    unsafe {
        disable_pic();
        APIC_INIT.store(true, Ordering::SeqCst);
    }
}
