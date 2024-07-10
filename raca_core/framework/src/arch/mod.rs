use smp::{BSP_ID, CPUS};

pub mod acpi;
pub mod apic;
pub mod gdt;
pub mod interrupts;
pub mod smp;

pub fn basic_init() {
    //gdt::init();
    //interrupts::init();
    let mut cpus = CPUS.lock();
    let bsp = cpus.bsp_cpu();
    bsp.init();
    let gdt_ptr_loaded = x86_64::instructions::tables::sgdt();
    let idt_ptr_loaded = x86_64::instructions::tables::sidt();

    let gdt_ptr: *const _ = &bsp.gdt;
    let idt_ptr: *const _ = &bsp.idt;

    log::info!("{:x} {:x}", gdt_ptr_loaded.base.as_u64(), gdt_ptr as u64);
    log::info!("{:x} {:x}", idt_ptr_loaded.base.as_u64(), idt_ptr as u64);
}

pub fn init_acpi() {
    acpi::init();
}

pub fn init_apic() {
    //apic::init();
    smp::init();
}
