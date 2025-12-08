use core::arch::global_asm;

use bit_field::BitField;
use loongarch64::{
    VirtAddr,
    registers::{
        BadVirtAddr, ExceptionEntry, ExceptionReturnAddress, ExceptionStatus, TimerIntClear,
    },
};

use crate::arch::{context::TrapFrame, timer::call_timer_callbacks};

global_asm!(include_str!("entry.asm"));

unsafe extern "C" {
    fn trap_entry();
}

pub fn init() {
    ExceptionEntry.write(trap_entry as *const () as u64);
}

#[unsafe(no_mangle)]
extern "C" fn trap_handler(frame: &mut TrapFrame) {
    let estat = ExceptionStatus.read();
    let ecode = estat.get_bits(16..=21);

    if ecode == 0 {
        if estat.get_bit(11) {
            call_timer_callbacks(frame);
            TimerIntClear.write(1);
        } else {
            log::warn!("Unknown interrupt!");
        }
        return;
    }

    let era = ExceptionReturnAddress.read();
    let badv = VirtAddr::new(BadVirtAddr.read());

    log::error!("Unhandled exception 0x{:x}!", ecode);
    log::error!("ERA(PC): 0x{:x} BADV: 0x{:x}", era, badv);

    panic!("Unrecoverable Exception");
}
