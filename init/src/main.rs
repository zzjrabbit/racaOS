#![no_std]
#![no_main]

use alloc::sync::Arc;
use limine::request::StackSizeRequest;
use zodiac::{arch::idle_loop, main, mem::VmSpace, println};

mod mem;
mod module;

extern crate alloc;

#[used]
#[unsafe(link_section = ".requests")]
static STACK_REQUEST: StackSizeRequest = StackSizeRequest::new().with_size(256 * 1024);

#[main]
pub fn main() {
    module::init().unwrap();
    idle_loop();
}

pub fn make_alloc() {
    let vm_space = Arc::new(VmSpace::new_user());
    vm_space.reader(0x10, 0).read_bytes(&mut []).unwrap();
}

#[zodiac::panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    println!("Panic occurred: {}", info);
    idle_loop();
}
