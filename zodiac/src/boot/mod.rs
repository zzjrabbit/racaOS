use core::slice::from_raw_parts;

use limine::{
    BaseRevision,
    request::{ExecutableAddressRequest, ExecutableFileRequest},
};

use crate::mem::VirtualAddress;

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::with_revision(4);

unsafe extern "Rust" {
    fn __zodiac_main() -> !;
}

#[unsafe(no_mangle)]
unsafe extern "C" fn kmain() -> ! {
    super::init();

    unsafe {
        __zodiac_main();
    }
}

#[used]
#[unsafe(link_section = ".requests")]
static KERNEL_REQUEST: ExecutableFileRequest = ExecutableFileRequest::new();

pub fn kernel_file() -> &'static [u8] {
    let response = KERNEL_REQUEST.get_response().unwrap();
    let file = response.file();
    unsafe { from_raw_parts(file.addr() as *const u8, file.size() as usize) }
}

#[used]
#[unsafe(link_section = ".requests")]
static KERNEL_BASE_REQUEST: ExecutableAddressRequest = ExecutableAddressRequest::new();

pub fn kernel_base() -> VirtualAddress {
    let response = KERNEL_BASE_REQUEST.get_response().unwrap();
    response.virtual_base() as VirtualAddress
}
