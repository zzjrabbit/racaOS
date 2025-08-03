use core::ptr::copy_nonoverlapping;

pub use error_code::*;

use crate::{
    hal::{cpu::Cpu, smp::CPUS},
    mem::VirtualAddress,
};

mod error_code;

#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct TrapFrame {
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub rbp: usize,
    pub rbx: usize,

    pub r11: usize,
    pub r10: usize,
    pub r9: usize,
    pub r8: usize,
    pub rsi: usize,
    pub rdi: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub rax: usize,

    pub int_num: usize,
    pub error_code: usize,

    // Pushed by CPU
    pub rip: usize,
    pub cs: usize,
    pub rflags: usize,

    // Pushed by CPU when Ring3->0
    pub rsp: usize,
    pub ss: usize,
}

impl TrapFrame {
    pub fn init_in(kernel_stack: &mut [u8], entry: usize, stack: usize, user_mode: bool) -> usize {
        let kernel_stack_end = kernel_stack.as_ptr() as usize + kernel_stack.len();
        log::info!("Kernel stack end: {:x}", kernel_stack_end);

        let mut frame = Self::default();

        frame.rip = entry;
        frame.rsp = if user_mode {
            stack
        } else {
            kernel_stack_end
        };
        frame.rflags = 0x200;
        CPUS.with_cpu_info(Cpu::current(), |cpu_info| {
            frame.cs = if user_mode {
                cpu_info.user_code_selector()
            } else {
                cpu_info.kernel_code_selector()
            };
            frame.ss = if user_mode {
                cpu_info.user_data_selector()
            } else {
                cpu_info.kernel_data_selector()
            };
        });

        unsafe {
            copy_nonoverlapping(
                &frame as *const Self,
                (kernel_stack_end - size_of::<Self>()) as *mut Self,
                1,
            );
        }
        kernel_stack_end - size_of::<Self>()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuException {
    ///  0 – #DE  Divide-by-zero error.
    DivisionError,
    ///  1 – #DB  Debug.
    Debug,
    ///  2 – NMI  Non-maskable interrupt.
    NonMaskableInterrupt,
    ///  3 – #BP  Breakpoint (INT3).
    BreakPoint,
    ///  4 – #OF  Overflow.
    Overflow,
    ///  5 – #BR  Bound-range exceeded.
    BoundRangeExceeded,
    ///  6 – #UD  Invalid or undefined opcode.
    InvalidOpcode,
    ///  7 – #NM  Device not available (FPU/MMX/SSE disabled).
    DeviceNotAvailable,
    ///  8 – #DF  Double fault (always pushes an error code of 0).
    DoubleFault,
    ///  9 – Coprocessor segment overrun (reserved on modern CPUs).
    CoprocessorSegmentOverrun,
    /// 10 – #TS  Invalid TSS.
    InvalidTss(SelectorErrorCode),
    /// 11 – #NP  Segment not present.
    SegmentNotPresent(SelectorErrorCode),
    /// 12 – #SS  Stack-segment fault.
    StackSegmentFault(SelectorErrorCode),
    /// 13 – #GP  General protection fault  
    GeneralProtectionFault(Option<SelectorErrorCode>),
    /// 14 – #PF  Page fault.
    PageFault(PageFaultErrorCode, VirtualAddress),
    // 15: Reserved
    /// 16 – #MF  x87 floating-point exception.
    X87FloatingPointException,
    /// 17 – #AC  Alignment check.  
    AlignmentCheck,
    /// 18 – #MC  Machine check.
    MachineCheck,
    /// 19 – #XM / #XF  SIMD/FPU floating-point exception.
    SIMDFloatingPointException,
    /// 20 – #VE  Virtualization exception.
    VirtualizationException,
    /// 21 – #CP  Control protection exception (CET).
    ControlProtectionException,
    // 22-27: Reserved
    /// 28 – #HV  Hypervisor injection exception.
    HypervisorInjectionException,
    /// 29 – #VC  VMM communication exception (SEV-ES GHCB).
    VMMCommunicationException,
    /// 30 – #SX  Security exception.
    SecurityException,
    // 31: Reserved
    /// Catch-all for reserved or undefined vector numbers.
    Reserved,
}

impl CpuException {
    pub(crate) fn new(trap_num: usize, error_code: usize) -> Option<Self> {
        let exception = match trap_num {
            0 => Self::DivisionError,
            1 => Self::Debug,
            2 => Self::NonMaskableInterrupt,
            3 => Self::BreakPoint,
            4 => Self::Overflow,
            5 => Self::BoundRangeExceeded,
            6 => Self::InvalidOpcode,
            7 => Self::DeviceNotAvailable,
            8 => {
                // A double fault will always generate an error code with a value of zero.
                debug_assert_eq!(error_code, 0);
                Self::DoubleFault
            }
            9 => Self::CoprocessorSegmentOverrun,
            10 => Self::InvalidTss(SelectorErrorCode::new_truncate(error_code as u64)),
            11 => Self::SegmentNotPresent(SelectorErrorCode::new_truncate(error_code as u64)),
            12 => Self::StackSegmentFault(SelectorErrorCode::new_truncate(error_code as u64)),
            13 => {
                let error_code = if error_code == 0 {
                    None
                } else {
                    Some(SelectorErrorCode::new_truncate(error_code as u64))
                };
                Self::GeneralProtectionFault(error_code)
            }
            14 => {
                let page_fault_addr = x86_64::registers::control::Cr2::read_raw() as usize;
                Self::PageFault(
                    PageFaultErrorCode::from_bits(error_code as u64).unwrap(),
                    page_fault_addr,
                )
            }
            // Reserved 15
            16 => Self::X87FloatingPointException,
            17 => Self::AlignmentCheck,
            18 => Self::MachineCheck,
            19 => Self::SIMDFloatingPointException,
            20 => Self::VirtualizationException,
            21 => Self::ControlProtectionException,
            // Reserved 22-27
            28 => Self::HypervisorInjectionException,
            29 => Self::VMMCommunicationException,
            30 => Self::SecurityException,
            // Reserved 31
            15 | 22..=27 | 31 => Self::Reserved,
            _ => return None,
        };

        Some(exception)
    }
}

