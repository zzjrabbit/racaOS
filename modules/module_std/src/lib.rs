#![no_std]

use core::mem::transmute;

#[repr(C)]
#[allow(dead_code)]
pub struct InfoStruct {
    _magic: [u8; 8],
    name: [u8; 8],
    kernel_function: fn(id: usize) -> usize,
}

fn default(_id: usize) -> usize {
    0
}

impl InfoStruct {
    pub const fn with_name(name: [u8; 8]) -> Self {
        Self {
            _magic: [b'K', b'e', b'r', b'n', b'e', b'l', b'M', b'O'],
            name: name,
            kernel_function: default,
        }
    }

    pub const fn get_name(&self) -> [u8; 8] {
        self.name
    }

    pub fn get_func<A>(&self, id: usize) -> fn(arg: A) -> usize
    where
        A: Sized,
    {
        unsafe { transmute(core::ptr::read_volatile(&self.kernel_function)(id)) }
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("{}", info);
    unsafe {
        core::arch::asm!("int 0");
    }
    loop {}
}
