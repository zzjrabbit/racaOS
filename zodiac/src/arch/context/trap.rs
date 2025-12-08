use crate::{arch::context::GeneralRegs, mem::VirtualAddress};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CpuExceptionInfo {
    pub code: u64,
    pub page_fault_addr: VirtualAddress,
    pub error_code: u64,
}

impl Default for CpuExceptionInfo {
    fn default() -> Self {
        CpuExceptionInfo {
            code: 0,
            page_fault_addr: 0,
            error_code: 0,
        }
    }
}

impl CpuExceptionInfo {
    /// Gets corresponding CPU exception.
    pub fn exception_code(&self) -> u64 {
        self.code
    }
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct TrapFrame {
    /// General registers
    pub general: GeneralRegs,
    /// Pre-exception Mode Information
    pub prmd: usize,
    /// Exception Return Address
    pub era: usize,
    /// Extended Unit Enable
    pub euen: usize,
}
