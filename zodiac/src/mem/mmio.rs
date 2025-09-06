use core::marker::PhantomData;

use crate::mem::{VirtualAddress, VirtualMemorySpace};

pub struct Mmio<T> {
    address: VirtualAddress,
    _marker: PhantomData<T>,
}

impl<T> Mmio<T> {
    pub fn new(address: VirtualAddress) -> Self {
        Self {
            address,
            _marker: PhantomData,
        }
    }

    pub fn read(&self) -> T {
        if let Err(_) = VirtualMemorySpace::new_kernel().query(self.address) {
            panic!("Trying to access an unmapped address.");
        };
        unsafe { core::ptr::read_volatile(self.address as *const T) }
    }

    pub fn write(&mut self, value: T) {
        if let Err(_) = VirtualMemorySpace::new_kernel().query(self.address) {
            panic!("Trying to access an unmapped address.");
        };
        unsafe { core::ptr::write_volatile(self.address as *mut T, value) }
    }
}
