pub mod context;
pub mod process;
pub mod scheduler;
pub mod stack;
pub mod thread;

pub use process::Process;
pub use scheduler::init;
pub use thread::Thread;

pub fn schedule() {
    unsafe {
        core::arch::asm!("int 0x20");
    }
}
