#![no_std]
#![no_main]
#![forbid(unsafe_code)]

use core::panic::PanicInfo;

use alloc::sync::Arc;
use spin::Mutex;
use zodiac::task::{start_schedule, Process, Thread, ThreadBuilder};

extern crate alloc;

mod heap;
mod scheduler;

fn thread_a() -> ! {
    loop {
        zodiac::print!("[doge]");
        for _ in 0..10000000 {
            core::hint::spin_loop();
        }
    }
}

static THREAD_A: Mutex<Option<Arc<Thread>>> = Mutex::new(None);

fn thread_b() -> ! {
    for _ in 0..10 {
        for _ in 0..10000000 {
            core::hint::spin_loop();
        }
    }
    
    log::info!("KILL!");
    
    THREAD_A.lock().as_ref().unwrap().kill();
    
    loop {}
}

#[zodiac::main]
pub fn main() {
    scheduler::init();
    log::info!("Zodiac Initialize done, entering kernel.");

    log::info!(
        "total time: {}s",
        zodiac::hal::timer::elapsed().as_secs_f64()
    );
    
    let thread = ThreadBuilder::default().entry(thread_a).kernel_mode().process(Process::kernel()).build().unwrap();
    thread.spawn();
    THREAD_A.lock().replace(thread.clone());
    
    let thread = ThreadBuilder::default().entry(thread_b).kernel_mode().process(Process::kernel()).build().unwrap();
    thread.spawn();
    
    start_schedule();

    loop {}
}

#[zodiac::panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
    log::error!("panic: {}", info);
    loop {}
}
