#![no_std]

#[repr(C)]
#[allow(dead_code)]
pub struct InfoStruct {
    name: &'static str,
}

impl InfoStruct {
    pub const fn with_name(name: &'static str) -> Self {
        Self {
            name,
        }
    }

    pub const fn get_name(&self) -> &'static str {
        self.name
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("{}", info);
    loop {}
}
