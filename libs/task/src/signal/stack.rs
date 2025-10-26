use bitflags::bitflags;
use ostd::{Pod, mm::Vaddr};

#[derive(Debug, Default)]
pub struct SignalStack {
    base: Vaddr,
    flags: SignalStackFlags,
    size: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SignalStackStatus {
    /// The stack is enabled but currently inactive.
    Inactive,
    /// The stack is currently active.
    Active,
    /// The stack is disabled.
    Disable,
}

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct SignalStackFlags: u32 {
        const SS_ONSTACK = 1 << 0;
        const SS_DISABLE = 1 << 1;
        const SS_AUTODISARM = 1 << 31;
    }
}

impl SignalStack {
    /// Creates a new signal stack.
    pub fn new(base: Vaddr, flags: SignalStackFlags, size: usize) -> Self {
        Self { base, flags, size }
    }

    /// Returns the lowest address of the signal stack.
    pub fn base(&self) -> Vaddr {
        self.base
    }

    /// Returns the signal stack flags as set by the user.
    pub fn flags(&self) -> SignalStackFlags {
        self.flags
    }

    /// Returns the current active status of the signal stack
    /// based on the given stack pointer.
    pub fn active_status(&self, sp: usize) -> SignalStackStatus {
        if self.size == 0 {
            return SignalStackStatus::Disable;
        }

        if self.contains(sp) {
            return SignalStackStatus::Active;
        }

        SignalStackStatus::Inactive
    }

    /// Returns the signal stack size.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns whether the given stack pointer is currently on the alternate signal stack.
    ///
    /// Note that if the `SS_AUTODISARM` flag is set,
    /// the alternate signal stack is automatically disarmed after use.
    /// In this case, even if `sp` lies within the stack range,
    /// we consider that the signal stack is not active.
    pub fn contains(&self, sp: usize) -> bool {
        if self.flags().contains(SignalStackFlags::SS_AUTODISARM) {
            return false;
        }

        // The stack grows down, so `self.base` is exclusive.
        self.base < sp && sp <= self.base + self.size
    }

    /// Resets the signal stack settings.
    pub(crate) fn reset(&mut self) {
        self.base = 0;
        self.size = 0;
        self.flags = SignalStackFlags::SS_DISABLE;
    }
}

#[derive(Debug, Clone, Copy, Pod, Default)]
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct stack_t {
    pub ss_sp: Vaddr,
    pub ss_flags: u32,
    pub ss_size: usize,
}

impl From<&SignalStack> for stack_t {
    fn from(value: &SignalStack) -> Self {
        Self {
            ss_sp: value.base,
            ss_flags: value.flags.bits(),
            ss_size: value.size,
        }
    }
}
