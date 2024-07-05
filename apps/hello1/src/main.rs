#![no_std]
#![no_main]
#![feature(naked_functions)]

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop{}
}

#[naked]
pub extern "C" fn syscall2(_rax: u64, _rdi: *const u8, _rsi: usize) -> usize {
    unsafe {
        core::arch::asm!(
            "mov rax, rdi",
            "mov rdi, rsi",
            "mov rsi, rdx",
            "syscall",
            "ret",
            options(noreturn)
        )
    }
}

pub fn write(buffer: *const u8, length: usize) -> usize {
    const WRITE_SYSCALL_NUMBER: u64 = 1;
    syscall2(WRITE_SYSCALL_NUMBER, buffer, length)
}

#[no_mangle]
pub fn _start() {
    write("[Hello]".as_ptr(),7);
    loop {
        unsafe {
            core::arch::asm!("nop");
        }
    }
}
