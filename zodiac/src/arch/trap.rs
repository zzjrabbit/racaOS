use bit_field::BitField;
use loongarch64::{
    VirtAddr,
    registers::{BadVirtAddr, ExceptionEntry, ExceptionReturnAddress, ExceptionStatus},
};

pub fn init() {
    ExceptionEntry.write(trap_handler as *const () as u64);
}

extern "C" fn trap_handler() {
    let estat = ExceptionStatus.read();
    let ecode = estat.get_bits(16..=21);

    let era = ExceptionReturnAddress.read();
    let badv = VirtAddr::new(BadVirtAddr.read());

    log::error!("Unhandled exception {}!", ecode);
    log::error!("ERA(PC): {:x} BADV: {:x}", era, badv);

    panic!("Unrecoverable Exception");
}
