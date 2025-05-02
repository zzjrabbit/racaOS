use x86_64::structures::port::{PortRead, PortWrite};

crate::kernel_object! {
    pub struct IoPort {
        port: u16 = port,
    }

    fn new(port: u16) {}
}

impl IoPort {
    pub fn read<T: PortRead>(&self) -> T {
        unsafe { x86_64::instructions::port::Port::new(self.port).read() }
    }

    pub fn write<T: PortWrite>(&self, value: T) {
        unsafe { x86_64::instructions::port::Port::new(self.port).write(value) }
    }
}
