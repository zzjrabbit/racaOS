use acpi::{AcpiError, AcpiTables, aml::Interpreter, platform::AcpiPlatform, sdt::spcr::Spcr};
use limine::request::RsdpRequest;
use spin::lazy::Lazy;

use crate::{
    acpi::handler::AcpiHandler,
    mem::{VirtualAddress, convert_virtual_to_physical},
};

mod handler;

#[used]
#[unsafe(link_section = ".requests")]
static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

pub static ACPI: Lazy<Acpi> = Lazy::new(|| init_acpi().unwrap());

pub struct Acpi {
    pub aml_engine: Interpreter<AcpiHandler>,
    pub serial_base: u64,
}

fn init_acpi() -> Result<Acpi, AcpiError> {
    let response = RSDP_REQUEST.get_response().unwrap();

    let platform_info = unsafe {
        let rsdp_address = response.address() as VirtualAddress;
        let tables = AcpiTables::from_rsdp(
            AcpiHandler,
            convert_virtual_to_physical(rsdp_address).into(),
        )?;
        AcpiPlatform::new(tables, AcpiHandler)?
    };

    let aml_engine = Interpreter::new_from_platform(&platform_info)?;

    let acpi_tables = &platform_info.tables;
    let spcr = acpi_tables.find_table::<Spcr>().unwrap();

    Ok(Acpi {
        aml_engine,
        serial_base: spcr.base_address().unwrap()?.address,
    })
}
