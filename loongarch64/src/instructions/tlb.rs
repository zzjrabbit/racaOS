use core::arch::asm;

use crate::VirtAddr;

pub fn flush(addr: VirtAddr) {
    unsafe {
        asm!(
            "invtlb 0x06, $zero, {}",
            in(reg) addr.as_u64()
        );
    }
}

pub fn flush_all() {
    unsafe {
        asm!("invtlb 0, $zero, $zero");
    }
}
