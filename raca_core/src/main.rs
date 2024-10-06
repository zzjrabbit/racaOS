#![no_std]
#![no_main]

use core::{ffi::CStr, panic::PanicInfo, slice};
use limine::{request::ModuleRequest, BaseRevision};
use raca_core::module::Module;

#[used]
#[link_section = ".requests"]
pub static BASE_REVISION: BaseRevision = BaseRevision::with_revision(1);

const fn hello_kernel_module_path() -> &'static [u8] {
    &[b'/', b'h', b'e', b'l', b'l', b'o', b'.', b'k', b'm', 0]
}

static HELLO_MODULE: limine::modules::InternalModule = limine::modules::InternalModule::new()
    .with_path(unsafe { CStr::from_bytes_with_nul_unchecked(hello_kernel_module_path()) });

#[used]
#[link_section = ".requests"]
static HELLO: ModuleRequest = ModuleRequest::new().with_internal_modules(&[&HELLO_MODULE]);

#[no_mangle]
pub extern "C" fn main() -> ! {
    raca_core::init();
    let module = HELLO.get_response().unwrap().modules()[0];
    let (ptr, size) = (module.addr(), module.size());
    let data = unsafe { slice::from_raw_parts(ptr, size as usize) };
    let module = Module::load(data);
    log::info!("module {} loaded", module.get_name());
    module.exec();

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{}", info);
    loop {
        x86_64::instructions::hlt();
    }
}
