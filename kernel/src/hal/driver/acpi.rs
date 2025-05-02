use crate::hal::ref_current_page_table;
use crate::mm::{MMUFlags, PhysicalMemory, VirtualMemory, VmMapping, page_count};
use acpi::fadt::Fadt;
use acpi::platform::interrupt::Apic;
use acpi::{AcpiHandler, AcpiTables, AmlTable, HpetInfo, PhysicalMapping};
use acpi::{InterruptModel, PciConfigRegions};
use alloc::alloc::Global;
use alloc::boxed::Box;
use alloc::sync::Arc;
use aml::{AmlContext, AmlName};
use core::ptr::NonNull;
use limine::request::RsdpRequest;
use spin::{Lazy, Mutex};
use x86_64::PhysAddr;
use x86_64::instructions::interrupts::disable;
use x86_64::instructions::port::{Port, PortWriteOnly};
use x86_64::structures::paging::{Page, PhysFrame, Size4KiB};

use super::super::mem::convert_physical_to_virtual;
use crate::hal::int::IntFrame;

#[used]
#[unsafe(link_section = ".requests")]
static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

pub static ACPI: Lazy<Acpi> = Lazy::new(|| {
    let response = RSDP_REQUEST.get_response().unwrap();

    let acpi_tables = unsafe {
        let rsdp_address = response.address();
        let acpi_tables = AcpiTables::from_rsdp(AcpiMemHandler, rsdp_address);
        Box::leak(Box::new(acpi_tables.unwrap()))
    };

    log::info!("Find ACPI tables successfully!");

    let platform_info = acpi_tables
        .platform_info()
        .expect("Failed to get platform info");

    let apic = match platform_info.interrupt_model {
        InterruptModel::Apic(apic) => apic,
        InterruptModel::Unknown => panic!("No APIC support, cannot continue!"),
        _ => panic!("ACPI does not have interrupt model info!"),
    };

    let pci_regions = PciConfigRegions::new(acpi_tables).expect("Failed to get PCI regions");
    let hpet_info = HpetInfo::new(acpi_tables).expect("Failed to get HPET info");

    let fadt = *acpi_tables.find_table::<Fadt>().unwrap();

    let pm1a = fadt.pm1a_control_block().unwrap();
    let mut pm1a_port = Port::<u16>::new(pm1a.address as u16);

    unsafe {
        if fadt.smi_cmd_port != 0
            && (fadt.acpi_enable == 0 || fadt.acpi_disable == 0)
            && pm1a_port.read() & 1 == 0
        {
            Port::new(fadt.smi_cmd_port as u16).write(fadt.acpi_enable);
            while pm1a_port.read() & 1 == 0 {}
        }
    }

    let dsdt = acpi_tables.dsdt().expect("DSDT Not Found");

    Acpi {
        apic,
        pci_regions,
        hpet_info,
        dsdt,
        fadt,
    }
});

/// # Panics
/// If it fails to register a handler for the timer interrupt, it will panic.
pub fn init() {
    let fadt = &ACPI.fadt;
    let vector = crate::hal::int::register_handler(poweroff_handler).unwrap();
    unsafe {
        super::apic::ioapic_add_entry(fadt.sci_interrupt as u8, vector as u8);
    }
}

pub struct Acpi<'a> {
    pub apic: Apic<'a, Global>,
    pub pci_regions: PciConfigRegions<'a, Global>,
    pub hpet_info: HpetInfo,
    pub fadt: Fadt,
    pub dsdt: AmlTable,
}

fn poweroff_handler(_frame: &mut IntFrame) {
    reboot();
}

/// # Panics
/// It panics when it fails to parse DSDT.
pub fn poweroff() {
    disable();
    let fadt = &ACPI.fadt;
    let dsdt = &ACPI.dsdt;

    let handler = Box::new(AmlHandlerImpl {});

    let dsdt_stream = unsafe {
        core::slice::from_raw_parts(
            convert_physical_to_virtual(PhysAddr::new(dsdt.address as u64)).as_ptr::<u8>(),
            dsdt.length as usize,
        )
    };
    let mut dsdt = AmlContext::new(handler, aml::DebugVerbosity::None);
    dsdt.parse_table(dsdt_stream).unwrap();

    let s5 = dsdt
        .invoke_method(
            &AmlName::from_str("\\_S5").unwrap(),
            aml::value::Args::EMPTY,
        )
        .unwrap();

    let slp_typa = match s5 {
        aml::AmlValue::Package(values) => values[0].clone(),
        _ => unreachable!(),
    }
    .as_integer(&dsdt)
    .unwrap();

    loop {
        unsafe {
            PortWriteOnly::new(fadt.pm1a_control_block().unwrap().address as u16)
                .write((slp_typa as u16) | (1 << 13));
        }
    }
}

/// # Panics
/// it panics when it can't parse FADT
pub fn reboot() {
    disable();
    let fadt = &ACPI.fadt;
    loop {
        unsafe {
            PortWriteOnly::new(fadt.reset_register().unwrap().address as u16)
                .write(fadt.reset_value);
        }
    }
}

#[derive(Clone)]
struct AcpiMemHandler;

impl AcpiHandler for AcpiMemHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let virtual_address = {
            let physical_address = PhysAddr::new(physical_address as u64);
            let virtual_address = convert_physical_to_virtual(physical_address);

            let size = page_count(size);
            let child = VirtualMemory::new(
                Page::<Size4KiB>::containing_address(virtual_address)
                    .start_address()
                    .as_u64() as usize,
                size,
                Arc::new(Mutex::new(ref_current_page_table())),
            );

            let frame = PhysFrame::<Size4KiB>::containing_address(physical_address);
            let start_address = frame.start_address().as_u64() as usize;
            let physical_memory = PhysicalMemory::new(start_address, size);
            let vm_mapping = Arc::new(VmMapping::new(
                MMUFlags::READ | MMUFlags::WRITE,
                child.clone(),
                physical_memory.clone(),
            ));

            let _ = vm_mapping.map();

            unsafe { NonNull::new_unchecked(virtual_address.as_u64() as *mut T) }
        };
        unsafe { PhysicalMapping::new(physical_address, virtual_address, size, size, self.clone()) }
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}

struct AmlHandlerImpl {}

impl aml::Handler for AmlHandlerImpl {
    fn read_u8(&self, address: usize) -> u8 {
        unsafe {
            core::ptr::read_volatile(
                convert_physical_to_virtual(PhysAddr::new(address as u64)).as_u64() as *const u8,
            )
        }
    }
    fn write_u8(&mut self, address: usize, value: u8) {
        unsafe {
            core::ptr::write_volatile(
                convert_physical_to_virtual(PhysAddr::new(address as u64)).as_u64() as *mut u8,
                value,
            );
        }
    }

    fn read_u16(&self, address: usize) -> u16 {
        unsafe {
            core::ptr::read_volatile(
                convert_physical_to_virtual(PhysAddr::new(address as u64)).as_u64() as *const u16,
            )
        }
    }
    fn write_u16(&mut self, address: usize, value: u16) {
        unsafe {
            core::ptr::write_volatile(
                convert_physical_to_virtual(PhysAddr::new(address as u64)).as_u64() as *mut u16,
                value,
            );
        }
    }

    fn read_u32(&self, address: usize) -> u32 {
        unsafe {
            core::ptr::read_volatile(
                convert_physical_to_virtual(PhysAddr::new(address as u64)).as_u64() as *const u32,
            )
        }
    }
    fn write_u32(&mut self, address: usize, value: u32) {
        unsafe {
            core::ptr::write_volatile(
                convert_physical_to_virtual(PhysAddr::new(address as u64)).as_u64() as *mut u32,
                value,
            );
        }
    }

    fn read_u64(&self, address: usize) -> u64 {
        unsafe {
            core::ptr::read_volatile(
                convert_physical_to_virtual(PhysAddr::new(address as u64)).as_u64() as *const u64,
            )
        }
    }
    fn write_u64(&mut self, address: usize, value: u64) {
        unsafe {
            core::ptr::write_volatile(
                convert_physical_to_virtual(PhysAddr::new(address as u64)).as_u64() as *mut u64,
                value,
            );
        }
    }

    fn read_io_u8(&self, _port: u16) -> u8 {
        unimplemented!()
    }
    fn read_io_u16(&self, _port: u16) -> u16 {
        unimplemented!()
    }
    fn read_io_u32(&self, _port: u16) -> u32 {
        unimplemented!()
    }

    fn write_io_u8(&self, _port: u16, _value: u8) {
        unimplemented!()
    }
    fn write_io_u16(&self, _port: u16, _value: u16) {
        unimplemented!()
    }
    fn write_io_u32(&self, _port: u16, _value: u32) {
        unimplemented!()
    }

    fn read_pci_u8(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16) -> u8 {
        unimplemented!()
    }
    fn read_pci_u16(
        &self,
        _segment: u16,
        _bus: u8,
        _device: u8,
        _function: u8,
        _offset: u16,
    ) -> u16 {
        unimplemented!()
    }
    fn read_pci_u32(
        &self,
        _segment: u16,
        _bus: u8,
        _device: u8,
        _function: u8,
        _offset: u16,
    ) -> u32 {
        unimplemented!()
    }

    fn write_pci_u8(
        &self,
        _segment: u16,
        _bus: u8,
        _device: u8,
        _function: u8,
        _offset: u16,
        _value: u8,
    ) {
        unimplemented!()
    }
    fn write_pci_u16(
        &self,
        _segment: u16,
        _bus: u8,
        _device: u8,
        _function: u8,
        _offset: u16,
        _value: u16,
    ) {
        unimplemented!()
    }
    fn write_pci_u32(
        &self,
        _segment: u16,
        _bus: u8,
        _device: u8,
        _function: u8,
        _offset: u16,
        _value: u32,
    ) {
        unimplemented!()
    }
}
