#![no_std]

use core::panic::PanicInfo;

#[doc(hidden)]
pub struct ModuleInfo {
    pub name: &'static str,
}

#[used]
#[unsafe(no_mangle)]
pub static _MODULE_INFO: ModuleInfo = ModuleInfo { name: "core-dylib" };

#[unsafe(no_mangle)]
pub fn init() {}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    unsafe {
        unsafe extern "Rust" {
            fn kernel_panic_handler(info: &PanicInfo) -> !;
        }

        kernel_panic_handler(info);
    }
}
