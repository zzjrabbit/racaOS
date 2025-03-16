use acpi::fadt::Fadt;
use acpi::platform::interrupt::Apic;
use acpi::{AcpiHandler, AcpiTables, AmlTable, HpetInfo, PhysicalMapping};
use acpi::{InterruptModel, PciConfigRegions};
use alloc::alloc::Global;
use alloc::boxed::Box;
use aml::{AmlContext, AmlName};
use core::ptr::NonNull;
use limine::request::RsdpRequest;
use spin::Lazy;
use x86_64::instructions::interrupts::disable;
use x86_64::instructions::port::{PortReadOnly, PortWriteOnly};
use x86_64::{PhysAddr, VirtAddr};

use super::super::mem::{convert_physical_to_virtual, convert_virtual_to_physical};
use crate::hal::int::IntFrame;

#[used]
#[unsafe(link_section = ".requests")]
static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

pub static ACPI: Lazy<Acpi> = Lazy::new(|| {
    let response = RSDP_REQUEST.get_response().unwrap();

    let acpi_tables = unsafe {
        let rsdp_address = VirtAddr::new(response.address() as u64);
        let physical_address = convert_virtual_to_physical(rsdp_address).as_u64();
        let acpi_tables = AcpiTables::from_rsdp(AcpiMemHandler, physical_address as usize);
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

    let fadt = acpi_tables.find_table::<Fadt>().unwrap();
    let dsdt = acpi_tables.dsdt().unwrap();

    if fadt.smi_cmd_port != 0 && (fadt.acpi_enable != 0 || fadt.acpi_disable != 0) {
        unsafe {
            PortWriteOnly::new(fadt.smi_cmd_port as u16).write(fadt.acpi_enable);
        }
    }
    while unsafe {
        PortReadOnly::<u16>::new(fadt.pm1a_control_block().unwrap().address as u16).read()
    } & 1
        == 0
    {}

    let event_register_len = fadt.pm1a_event_block().unwrap().bit_width / 16;

    unsafe {
        PortWriteOnly::new(
            fadt.pm1a_event_block().unwrap().address as u16 + event_register_len as u16,
        )
        .write(1_u16 << 8);
    }

    if let Ok(Some(pm1b)) = fadt.pm1b_event_block() {
        let len = pm1b.bit_width / 16;
        unsafe {
            PortWriteOnly::new(pm1b.address as u16 + len as u16).write(1_u16 << 8);
        }
    }

    Acpi {
        apic,
        pci_regions,
        hpet_info,
        dsdt,
        fadt: *fadt,
    }
});

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
                .write((slp_typa as u32) | (1 << 13));
        }
    }
}

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
