use core::fmt;

use bit_field::BitField;
use bitflags::bitflags;

bitflags! {
    /// Describes an page fault error code.
    ///
    /// This structure is defined by the following manual sections:
    ///   * AMD Volume 2: 8.4.2
    ///   * Intel Volume 3A: 4.7
    #[repr(transparent)]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct PageFaultErrorCode: u64 {
        /// If this flag is set, the page fault was caused by a page-protection violation,
        /// else the page fault was caused by a not-present page.
        const PROTECTION_VIOLATION = 1;

        /// If this flag is set, the memory access that caused the page fault was a write.
        /// Else the access that caused the page fault is a memory read. This bit does not
        /// necessarily indicate the cause of the page fault was a read or write violation.
        const CAUSED_BY_WRITE = 1 << 1;

        /// If this flag is set, an access in user mode (CPL=3) caused the page fault. Else
        /// an access in supervisor mode (CPL=0, 1, or 2) caused the page fault. This bit
        /// does not necessarily indicate the cause of the page fault was a privilege violation.
        const USER_MODE = 1 << 2;

        /// If this flag is set, the page fault is a result of the processor reading a 1 from
        /// a reserved field within a page-translation-table entry.
        const MALFORMED_TABLE = 1 << 3;

        /// If this flag is set, it indicates that the access that caused the page fault was an
        /// instruction fetch.
        const INSTRUCTION_FETCH = 1 << 4;

        /// If this flag is set, it indicates that the page fault was caused by a protection key.
        const PROTECTION_KEY = 1 << 5;

        /// If this flag is set, it indicates that the page fault was caused by a shadow stack
        /// access.
        const SHADOW_STACK = 1 << 6;

        /// If this flag is set, it indicates that the page fault was caused by SGX access-control
        /// requirements (Intel-only).
        const SGX = 1 << 15;

        /// If this flag is set, it indicates that the page fault is a result of the processor
        /// encountering an RMP violation (AMD-only).
        const RMP = 1 << 31;
    }
}

/// Describes an error code referencing a segment selector.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SelectorErrorCode {
    flags: u64,
}

impl SelectorErrorCode {
    /// Create a SelectorErrorCode. Returns None is any of the reserved bits (16-64) are set.
    pub const fn new(value: u64) -> Option<Self> {
        if value > u16::MAX as u64 {
            None
        } else {
            Some(Self { flags: value })
        }
    }

    /// Create a new SelectorErrorCode dropping any reserved bits (16-64).
    pub const fn new_truncate(value: u64) -> Self {
        Self {
            flags: (value as u16) as u64,
        }
    }

    /// If true, indicates that the exception occurred during delivery of an event
    /// external to the program, such as an interrupt or an earlier exception.
    pub fn external(&self) -> bool {
        self.flags.get_bit(0)
    }

    /// The descriptor table this error code refers to.
    pub fn descriptor_table(&self) -> DescriptorTable {
        match self.flags.get_bits(1..3) {
            0b00 => DescriptorTable::Gdt,
            0b01 => DescriptorTable::Idt,
            0b10 => DescriptorTable::Ldt,
            0b11 => DescriptorTable::Idt,
            _ => unreachable!(),
        }
    }

    /// The index of the selector which caused the error.
    pub fn index(&self) -> u64 {
        self.flags.get_bits(3..16)
    }

    /// If true, the #SS or #GP has returned zero as opposed to a SelectorErrorCode.
    pub fn is_null(&self) -> bool {
        self.flags == 0
    }
}

impl fmt::Debug for SelectorErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = f.debug_struct("Selector Error");
        s.field("external", &self.external());
        s.field("descriptor table", &self.descriptor_table());
        s.field("index", &self.index());
        s.finish()
    }
}

/// The possible descriptor table values.
///
/// Used by the [`SelectorErrorCode`] to indicate which table caused the error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DescriptorTable {
    /// Global Descriptor Table.
    Gdt,
    /// Interrupt Descriptor Table.
    Idt,
    /// Logical Descriptor Table.
    Ldt,
}
