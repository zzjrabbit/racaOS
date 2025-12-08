use crate::arch::context::{CpuExceptionInfo, GeneralRegs};

/// Saved registers on a trap.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub(in crate::arch) struct RawUserContext {
    /// General registers
    pub(in crate::arch) general: GeneralRegs,
    /// Pre-exception Mode Information
    pub(in crate::arch) prmd: usize,
    /// Exception Return Address
    pub(in crate::arch) era: usize,
    /// Extended Unit Enable
    pub(in crate::arch) euen: usize,
}

impl Default for RawUserContext {
    fn default() -> Self {
        Self {
            general: GeneralRegs::default(),
            prmd: 0b111, // User mode, enable interrupt
            era: 0,
            euen: 0,
        }
    }
}

impl RawUserContext {
    pub(in crate::arch) fn run(&mut self) {
        unsafe extern "C" {
            fn run_user(regs: &mut RawUserContext);
        }
        unsafe {
            run_user(self);
        }
    }
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct UserContext {
    user_context: RawUserContext,
    trap: u64,
    cpu_exception_info: Option<CpuExceptionInfo>,
}

impl Default for UserContext {
    fn default() -> Self {
        UserContext {
            user_context: RawUserContext::default(),
            trap: 0,
            cpu_exception_info: None,
        }
    }
}

impl UserContext {
    /// Returns a reference to the general registers.
    pub fn general_regs(&self) -> &GeneralRegs {
        &self.user_context.general
    }

    /// Returns a mutable reference to the general registers
    pub fn general_regs_mut(&mut self) -> &mut GeneralRegs {
        &mut self.user_context.general
    }

    /// Returns the trap information.
    pub fn take_exception(&mut self) -> Option<CpuExceptionInfo> {
        self.cpu_exception_info.take()
    }

    /// Sets the thread-local storage pointer.
    pub fn set_tls_pointer(&mut self, tls: usize) {
        self.set_tp(tls)
    }

    /// Gets the thread-local storage pointer.
    pub fn tls_pointer(&self) -> usize {
        self.tp()
    }

    /// Activates the thread-local storage pointer for the current task.
    pub fn activate_tls_pointer(&self) {
        // In LoongArch, `tp` will be loaded at `UserContext::execute`, so it does not need to be
        // activated in advance.
    }

    /// Enables floating-point unit.
    pub fn enable_fpu(&mut self) {
        self.user_context.euen = 0x1;
    }
}

macro_rules! cpu_context_impl_getter_setter {
    ( $( [ $field: ident, $setter_name: ident] ),*) => {
        impl UserContext {
            $(
                #[doc = concat!("Gets the value of ", stringify!($field))]
                #[inline(always)]
                pub fn $field(&self) -> usize {
                    self.user_context.general.$field
                }

                #[doc = concat!("Sets the value of ", stringify!($field))]
                #[inline(always)]
                pub fn $setter_name(&mut self, $field: usize) {
                    self.user_context.general.$field = $field;
                }
            )*
        }
    };
}

cpu_context_impl_getter_setter!(
    [ra, set_ra],
    [tp, set_tp],
    [sp, set_sp],
    [a0, set_a0],
    [a1, set_a1],
    [a2, set_a2],
    [a3, set_a3],
    [a4, set_a4],
    [a5, set_a5],
    [a6, set_a6],
    [a7, set_a7],
    [t0, set_t0],
    [t1, set_t1],
    [t2, set_t2],
    [t3, set_t3],
    [t4, set_t4],
    [t5, set_t5],
    [t6, set_t6],
    [t7, set_t7],
    [t8, set_t8],
    [r21, set_r21],
    [fp, set_fp],
    [s0, set_s0],
    [s1, set_s1],
    [s2, set_s2],
    [s3, set_s3],
    [s4, set_s4],
    [s5, set_s5],
    [s6, set_s6],
    [s7, set_s7],
    [s8, set_s8]
);
