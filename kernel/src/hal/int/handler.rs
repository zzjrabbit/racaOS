use alloc::{
    collections::btree_map::BTreeMap,
    vec::Vec,
};
use spin::{Lazy, Mutex, RwLock};
use x86_64::{registers::control::Cr2, structures::idt::PageFaultErrorCode};

use crate::hal::driver::apic::LAPIC;

pub type InterruptHandler = fn(frame: &mut IntFrame);

pub static INTERRUPT_HANDLERS: RwLock<BTreeMap<usize, Option<InterruptHandler>>> =
    RwLock::new(BTreeMap::new());

pub fn register_handler(handler: InterruptHandler) -> Option<usize> {
    let mut handlers = INTERRUPT_HANDLERS.write();

    let int = handlers.len() + super::INTERRUPT_OFFSET;
    if int <= 0xff {
        handlers.insert(int, Some(handler));
        Some(int)
    } else {
        None
    }
}

pub fn allocate_interrupt() -> Option<usize> {
    let mut handlers = INTERRUPT_HANDLERS.write();

    let int = handlers.len() + super::INTERRUPT_OFFSET;
    if int <= 0xff {
        handlers.insert(int, None);
        Some(int)
    } else {
        None
    }
}

pub static INTERRUPT_COUNT: Lazy<Mutex<Vec<usize>>> = Lazy::new(|| Mutex::new(alloc::vec![0;256]));

#[unsafe(no_mangle)]
pub extern "C" fn rust_entry(frame: &mut IntFrame) {
    if frame.int_num >= super::INTERRUPT_OFFSET {
        crate::hal::driver::apic::end_of_interrupt();

        let handlers = INTERRUPT_HANDLERS.read();
        let Some(Some(handler)) = handlers.get(&frame.int_num) else {
            INTERRUPT_COUNT.lock()[frame.int_num] += 1;
            return;
        };
        handler(frame);

        return;
    }

    match frame.int_num {
        0 => division_zero(frame),
        11 => segment_not_present(frame),
        13 => general_protection_fault(frame),
        6 => invalid_opcode(frame),
        3 => breakpoint(frame),
        8 => double_fault(frame),
        14 => page_fault(frame),
        _ => {}
    }

    loop {
        x86_64::instructions::hlt();
    }
}

fn division_zero(frame: &IntFrame) {
    log::error!("Exception: Division Zero\n{:#x?}", frame);
    panic!("Unrecoverable fault occured, halting!");
}

fn segment_not_present(frame: &IntFrame) {
    log::error!("Exception: Segment Not Present\n{:#x?}", frame);
    panic!("Unrecoverable fault occured, halting!");
}

fn general_protection_fault(frame: &IntFrame) {
    log::error!("Exception: General Protection Fault\n{:#x?}", frame);
    x86_64::instructions::hlt();
}

fn invalid_opcode(frame: &IntFrame) {
    log::error!("Exception: Invalid Opcode\n{:#x?}", frame);
    x86_64::instructions::hlt();
}

fn breakpoint(frame: &IntFrame) {
    log::debug!("Exception: Breakpoint\n{:#x?}", frame);
}

fn double_fault(frame: &IntFrame) -> ! {
    log::error!("Exception: Double Fault\n{:#x?}", frame);
    panic!("Unrecoverable fault occured, halting!");
}

fn page_fault(frame: &IntFrame) {
    log::warn!(
        "Exception: Page Fault From CPU {}\n{:#x?}",
        unsafe { LAPIC.lock().id() },
        frame
    );
    log::warn!(
        "Error Code: {:?}",
        PageFaultErrorCode::from_bits_retain(frame.error_code as u64)
    );
    match Cr2::read() {
        Ok(address) => {
            log::warn!("Fault Address: {:#x}", address);
        }
        Err(error) => {
            log::warn!("Invalid virtual address: {:?}", error);
        }
    }
    panic!("Cannot recover from page fault, halting!");
}

#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct IntFrame {
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub rbp: usize,
    pub rbx: usize,

    pub r11: usize,
    pub r10: usize,
    pub r9: usize,
    pub r8: usize,
    pub rsi: usize,
    pub rdi: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub rax: usize,

    pub int_num: usize,
    pub error_code: usize,

    // Pushed by CPU
    pub rip: usize,
    pub cs: usize,
    pub rflags: usize,

    // Pushed by CPU when Ring3->0
    pub rsp: usize,
    pub ss: usize,
}
