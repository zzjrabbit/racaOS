use limine::{mp::Cpu, request::FramebufferRequest};
use x86_64::registers::control::{Cr0, Cr0Flags, Cr4, Cr4Flags};

use crate::hal::smp::{BSP_LAPIC_ID, CPUS};

pub mod context;
pub mod cpu;
pub mod device;
pub mod kernel;
pub mod mem;
mod smp;
pub mod timer;
pub mod trap;

#[used]
#[unsafe(link_section = ".requests")]
static FB_REQUEST: FramebufferRequest = FramebufferRequest::new();

fn fb() -> (usize, &'static mut [u8]) {
    let fb = FB_REQUEST
        .get_response()
        .unwrap()
        .framebuffers()
        .next()
        .unwrap();

    let address = fb.addr();
    (fb.width() as usize, unsafe {
        core::slice::from_raw_parts_mut(address, fb.height() as usize * fb.width() as usize * 4)
    })
}

pub fn without_interrupts<R>(function: impl Fn() -> R) -> R {
    x86_64::instructions::interrupts::without_interrupts(function)
}

pub fn enable_interrupts() {
    x86_64::instructions::interrupts::enable();
}

pub fn disable_interrupts() {
    x86_64::instructions::interrupts::disable();
}

pub fn cpu_num() -> usize {
    smp::CPUS.len()
}

#[macro_export]
macro_rules! switch_stack {
    ($stack_pointer: expr) => {
        unsafe {
            core::arch::asm!(
                "mov rsp, {0}",
                in(reg) $stack_pointer,
            );
        }
    };
}

#[macro_export]
macro_rules! return_from_int {
    () => {
        unsafe {
            core::arch::asm!(
                "pop r15",
                "pop r14",
                "pop r13",
                "pop r12",
                "pop rbp",
                "pop rbx",
                
                "pop r11",
                "pop r10",
                "pop r9",
                "pop r8",
                "pop rsi",
                "pop rdi",
                "pop rdx",
                "pop rcx",
                "pop rax",
                
                "add rsp, 16",
                "iretq",
                options(noreturn),
            );
        }
    };
}

pub fn init() {
    smp::CPUS.load(*BSP_LAPIC_ID);
    trap::idt::init();
    init_sse();
    smp::CPUS.init_ap();
    kernel::init();
    timer::init();

    let (width, fb) = fb();

    let color = 0xff - *BSP_LAPIC_ID as u8;
    let start_line = *BSP_LAPIC_ID as usize * 6;
    let start_pixel = start_line * width;
    for i in 0..3 * width {
        let pos = (start_pixel + i) * 4;
        fb[pos + 0] = color;
        fb[pos + 1] = color;
        fb[pos + 2] = color;
        fb[pos + 3] = color;
    }
}

pub fn init_sse() {
    let mut cr0 = Cr0::read();
    cr0.remove(Cr0Flags::EMULATE_COPROCESSOR);
    cr0.insert(Cr0Flags::MONITOR_COPROCESSOR);
    unsafe { Cr0::write(cr0) };

    let mut cr4 = Cr4::read();
    cr4.insert(Cr4Flags::OSFXSR);
    cr4.insert(Cr4Flags::OSXMMEXCPT_ENABLE);
    unsafe { Cr4::write(cr4) };
}

unsafe extern "C" fn ap_entry(smp_info: &Cpu) -> ! {
    disable_interrupts();
    CPUS.load(smp_info.lapic_id);
    trap::idt::init();

    init_sse();

    kernel::ap_init();
    timer::ap_init();

    log::debug!("Application Processor {} started", smp_info.id);

    let (width, fb) = fb();

    let color = 0xff - smp_info.lapic_id as u8;
    let start_line = smp_info.lapic_id as usize * 6;
    let start_pixel = start_line * width;
    for i in 0..3 * width {
        let pos = (start_pixel + i) * 4;
        fb[pos + 0] = color;
        fb[pos + 1] = color;
        fb[pos + 2] = color;
        fb[pos + 3] = color;
    }

    crate::task::ap_init();

    loop {
        x86_64::instructions::hlt();
    }
}
