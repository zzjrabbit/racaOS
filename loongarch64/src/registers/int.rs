use bit_field::BitField;

use crate::define_csr;

define_csr!(ExceptionEntry, 0xc);
define_csr!(ExceptionConfig, 0x4);
define_csr!(ExceptionStatus, 0x5);
define_csr!(ExceptionReturnAddress, 0x6);
define_csr!(BadVirtAddr, 0x7);

define_csr!(TimerConfig, 0x41);
define_csr!(TimerIntClear, 0x44);

#[derive(Clone, Copy)]
pub struct TimerConfigBuilder {
    value: u64,
}

impl TimerConfigBuilder {
    pub fn new() -> Self {
        TimerConfigBuilder { value: 0 }
    }

    pub fn set_enabled(&mut self, enabled: bool) -> Self {
        self.value.set_bit(0, enabled);
        *self
    }

    pub fn set_periodic(&mut self, periodic: bool) -> Self {
        self.value.set_bit(1, periodic);
        *self
    }

    pub fn initial_value(&mut self, value: u64) -> Self {
        self.value.set_bits(2..64, value);
        *self
    }
}

impl TimerConfigBuilder {
    pub fn done(self) {
        TimerConfig.write(self.value);
    }
}
