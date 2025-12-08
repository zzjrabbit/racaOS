use bit_field::BitField;

use crate::{PrivilegeLevel, define_csr};

define_csr!(CurrentModeInfo, 0x0);

#[derive(Clone, Copy)]
pub struct CurrentModeInfoBuilder {
    value: u64,
}

impl CurrentModeInfoBuilder {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn new_with(value: u64) -> Self {
        Self { value }
    }

    pub fn done(self) {
        CurrentModeInfo.write(self.value);
    }
}

impl CurrentModeInfoBuilder {
    pub fn set_privilege(&mut self, privilege: PrivilegeLevel) -> Self {
        self.value |= privilege as u64;
        *self
    }

    pub fn enable_global_interrupt(&mut self) -> Self {
        self.value.set_bit(2, true);
        *self
    }

    pub fn disable_global_interrupt(&mut self) -> Self {
        self.value.set_bit(2, false);
        *self
    }

    pub fn enable_direct_memory_access(&mut self) -> Self {
        self.value.set_bit(3, true);
        *self
    }

    pub fn disable_direct_memory_access(&mut self) -> Self {
        self.value.set_bit(3, false);
        *self
    }

    pub fn enable_paging(&mut self) -> Self {
        self.value.set_bit(4, true);
        *self
    }

    pub fn disable_paging(&mut self) -> Self {
        self.value.set_bit(4, false);
        *self
    }
}
