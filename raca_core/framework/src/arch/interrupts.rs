use spin::Lazy;
use x86_64::instructions::port::PortReadOnly;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::VirtAddr;

use super::gdt::DOUBLE_FAULT_IST_INDEX;
use crate::arch::apic::get_lapic_id;
use crate::arch::apic::LAPIC;
use crate::task::scheduler::SCHEDULER;

const INTERRUPT_INDEX_OFFSET: u8 = 32;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = INTERRUPT_INDEX_OFFSET,
    ApicError,
    ApicSpurious,
    Keyboard,
    Mouse,
}

pub fn build_idt() -> InterruptDescriptorTable {
    let mut idt = InterruptDescriptorTable::new();

    idt.breakpoint.set_handler_fn(breakpoint);
    idt.segment_not_present.set_handler_fn(segment_not_present);
    idt.invalid_opcode.set_handler_fn(invalid_opcode);
    idt.page_fault.set_handler_fn(page_fault);
    idt.general_protection_fault
        .set_handler_fn(general_protection_fault);

    idt[InterruptIndex::Timer as u8].set_handler_fn(timer_interrupt);
    idt[InterruptIndex::ApicError as u8].set_handler_fn(lapic_error);
    idt[InterruptIndex::ApicSpurious as u8].set_handler_fn(spurious_interrupt);
    idt[InterruptIndex::Keyboard as u8].set_handler_fn(keyboard_interrupt);
    idt[InterruptIndex::Mouse as u8].set_handler_fn(mouse_interrupt);

    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX);
    }

    idt
}

pub static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    idt.breakpoint.set_handler_fn(breakpoint);
    idt.segment_not_present.set_handler_fn(segment_not_present);
    idt.invalid_opcode.set_handler_fn(invalid_opcode);
    idt.page_fault.set_handler_fn(page_fault);
    idt.general_protection_fault
        .set_handler_fn(general_protection_fault);

    idt[InterruptIndex::Timer as u8].set_handler_fn(timer_interrupt);
    idt[InterruptIndex::ApicError as u8].set_handler_fn(lapic_error);
    idt[InterruptIndex::ApicSpurious as u8].set_handler_fn(spurious_interrupt);
    idt[InterruptIndex::Keyboard as u8].set_handler_fn(keyboard_interrupt);
    idt[InterruptIndex::Mouse as u8].set_handler_fn(mouse_interrupt);

    unsafe {
        idt.double_fault
            .set_handler_fn(double_fault)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX);
    }

    return idt;
});

pub fn init() {
    IDT.load();
}

fn get_ap_id() -> u32 {
    unsafe { LAPIC.try_get().unwrap().lock().id() }
}

//#[naked]
pub extern "x86-interrupt" fn timer_interrupt(_frame: InterruptStackFrame) {
    fn timer_handler(context: VirtAddr) -> VirtAddr {
        let addr = SCHEDULER.write().schedule(context);
        super::apic::end_of_interrupt();
        addr
    }

    unsafe {
        core::arch::asm!(
            "cli",
            crate::push_context!(),
            "mov rdi, rsp",
            "call {timer_handler}",
            "mov rsp, rax",
            crate::pop_context!(),
            "sti",
            "iretq",
            timer_handler = sym timer_handler,
            options(noreturn)
        );
    }
}

pub extern "x86-interrupt" fn lapic_error(_frame: InterruptStackFrame) {
    log::error!("Local APIC error!");
    super::apic::end_of_interrupt();
}

pub extern "x86-interrupt" fn spurious_interrupt(_frame: InterruptStackFrame) {
    log::debug!("Received spurious interrupt!");
    super::apic::end_of_interrupt();
}

pub extern "x86-interrupt" fn segment_not_present(frame: InterruptStackFrame, error_code: u64) {
    log::error!("Exception: Segment Not Present\n{:#?}", frame);
    log::error!("Error Code: {:#x}", error_code);
    panic!("Unrecoverable fault occured, halting!");
}

pub extern "x86-interrupt" fn general_protection_fault(
    frame: InterruptStackFrame,
    error_code: u64,
) {
    log::error!("Processor {}!", get_lapic_id());
    log::error!("Exception: General Protection Fault\n{:#?}", frame);
    log::error!("Error Code: {:#x}", error_code);
    x86_64::instructions::hlt();
}

pub extern "x86-interrupt" fn invalid_opcode(frame: InterruptStackFrame) {
    log::error!(
        "Exception: Processor {} Invalid Opcode\n{:#?}",
        get_ap_id(),
        frame
    );
    //loop {}
    x86_64::instructions::hlt();
}

pub extern "x86-interrupt" fn breakpoint(frame: InterruptStackFrame) {
    log::debug!("Exception: Breakpoint\n{:#?}", frame);
}

pub extern "x86-interrupt" fn double_fault(frame: InterruptStackFrame, error_code: u64) -> ! {
    log::error!("Exception: Double Fault\n{:#?}", frame);
    log::error!("Error Code: {:#x}", error_code);
    panic!("Unrecoverable fault occured, halting!");
}

pub extern "x86-interrupt" fn keyboard_interrupt(_frame: InterruptStackFrame) {
    let scancode: u8 = unsafe { PortReadOnly::new(0x60).read() };
    crate::drivers::keyboard::add_scancode(scancode);
    super::apic::end_of_interrupt();
}

pub extern "x86-interrupt" fn mouse_interrupt(_frame: InterruptStackFrame) {
    let packet = unsafe { PortReadOnly::new(0x60).read() };
    crate::drivers::mouse::MOUSE.lock().process_packet(packet);
    super::apic::end_of_interrupt();
}

pub extern "x86-interrupt" fn page_fault(
    frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    log::warn!("Exception: Processor {} Page Fault", get_ap_id());
    log::warn!("Error Code: {:#x}", error_code);
    match Cr2::read() {
        Ok(address) => {
            log::warn!("Fault Address: {:#x}", address);
        }
        Err(error) => {
            log::warn!("Invalid virtual address: {:?}", error);
        }
    }
    log::warn!("{:#?}", frame);
    x86_64::instructions::hlt();
}
