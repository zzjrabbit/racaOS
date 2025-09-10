pub use error_code::*;
use x86_64::registers::rflags::RFlags;

use crate::{
    hal::{cpu::Cpu, smp::CPUS, trap::syscall::syscall_return},
    mem::VirtualAddress,
    task::ReturnReason,
    trap::IRQ_MANAGER,
};

mod error_code;

#[derive(Debug, Clone)]
pub(crate) struct RawUserContext(TrapFrame);

impl RawUserContext {
    pub fn new(entry: usize, stack: usize) -> Self {
        let mut frame = TrapFrame::default();
        frame.rsp = stack;
        frame.rip = entry;

        frame.rflags = 0x200;
        CPUS.with_cpu_info(Cpu::current(), |cpu_info| {
            frame.cs = cpu_info.user_code_selector();
            frame.ss = cpu_info.user_data_selector();
            log::info!("cs: {:x}, ss: {:x}", frame.cs, frame.ss);
        });

        Self(frame)
    }

    fn error_code(&self) -> usize {
        self.0.error_code
    }

    fn int_num(&self) -> usize {
        self.0.int_num
    }

    fn run(&mut self) {
        unsafe {
            syscall_return(&mut self.0);
        }
    }

    pub fn execute<F>(&mut self, mut has_kernel_event: F) -> ReturnReason
    where
        F: FnMut() -> bool,
    {
        self.0.rflags |= (RFlags::INTERRUPT_FLAG | RFlags::ID).bits() as usize;

        loop {
            self.run();

            let exception = CpuException::new(self.int_num(), self.error_code());

            match exception {
                Some(exception) => return ReturnReason::Exception(exception),
                None if self.int_num() == usize::MAX => return ReturnReason::Syscall,
                None => {
                    IRQ_MANAGER.handle_irq(&mut self.0);
                }
            }

            if has_kernel_event() {
                return ReturnReason::KernelEvent;
            }
        }
    }

    pub fn trap_frame(&mut self) -> &mut TrapFrame {
        &mut self.0
    }
}

/// When interrupt occurs, this structure will be pushed into the stack, either by CPU or by assembly code in Zodiac.
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
    pub rsp: usize,
    pub ss: usize,
}

impl TrapFrame {
    pub(crate) fn init(&mut self, kernel_stack: &[u8], entry: usize) {
        let kernel_stack_end = kernel_stack.as_ptr() as usize + kernel_stack.len();
        log::info!("Kernel stack end: {:x}", kernel_stack_end);

        self.rip = entry;
        self.rsp = kernel_stack_end;
        self.rflags = 0x200;
        CPUS.with_cpu_info(Cpu::current(), |cpu_info| {
            self.cs = cpu_info.kernel_code_selector();
            self.ss = cpu_info.kernel_data_selector();
        });
    }

    #[allow(dead_code)]
    pub(crate) fn set_stack(&mut self, stack: VirtualAddress) {
        self.rsp = stack;
    }

    pub fn syscall_index(&self) -> usize {
        self.rax
    }

    pub fn syscall_arguments(&self) -> [usize; 6] {
        [self.rdi, self.rsi, self.rdx, self.r10, self.r8, self.r9]
    }

    pub fn set_return_value(&mut self, value: usize) {
        self.rax = value;
    }
}

/// Cpu exceptions.
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
