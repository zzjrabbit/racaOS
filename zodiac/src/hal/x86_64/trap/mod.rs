use x86_64::registers::control::Cr2;

use crate::mem::{VirtualAddress, VirtualMemory};

pub(super) mod gdt;
pub(super) mod idt;

#[unsafe(no_mangle)]
pub extern "C" fn rust_entry(frame: &mut TrapFrame) {
    match frame.int_num {
        0xe => match Cr2::read() {
            Ok(address) => {
                VirtualMemory::kernel()
                    .handle_page_fault(address.as_u64() as VirtualAddress)
                    .unwrap();
            }
            Err(error) => {
                log::error!("Invalid virtual address: {:?}", error);
                loop {}
            }
        },
        _ => {
            log::info!("Interrupt: {:x?}", frame);
            loop {}
        }
    }
}

#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct TrapFrame {
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
