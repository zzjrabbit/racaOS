use ostd::Pod;

use core::{fmt::LowerHex, ops};

use crate::Signal;

pub type SignalMask = SignalSet;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Pod)]
#[repr(C)]
pub struct SignalSet {
    bits: u64,
}

impl From<Signal> for SignalSet {
    fn from(signal: Signal) -> Self {
        let idx = signal - Signal::MIN_STD_SIGNAL;
        Self { bits: 1_u64 << u8::from(idx) }
    }
}

impl From<u64> for SignalSet {
    fn from(bits: u64) -> Self {
        SignalSet { bits }
    }
}

impl From<SignalSet> for u64 {
    fn from(set: SignalSet) -> u64 {
        set.bits
    }
}

impl<T: Into<SignalSet>> ops::BitAnd<T> for SignalSet {
    type Output = Self;

    fn bitand(self, rhs: T) -> Self {
        SignalSet {
            bits: self.bits & rhs.into().bits,
        }
    }
}

impl<T: Into<SignalSet>> ops::BitAndAssign<T> for SignalSet {
    fn bitand_assign(&mut self, rhs: T) {
        self.bits &= rhs.into().bits;
    }
}

impl<T: Into<SignalSet>> ops::BitOr<T> for SignalSet {
    type Output = Self;

    fn bitor(self, rhs: T) -> Self {
        SignalSet {
            bits: self.bits | rhs.into().bits,
        }
    }
}

impl<T: Into<SignalSet>> ops::BitOrAssign<T> for SignalSet {
    fn bitor_assign(&mut self, rhs: T) {
        self.bits |= rhs.into().bits;
    }
}

#[expect(clippy::suspicious_arithmetic_impl)]
impl<T: Into<SignalSet>> ops::Add<T> for SignalSet {
    type Output = Self;

    fn add(self, rhs: T) -> Self {
        SignalSet {
            bits: self.bits | rhs.into().bits,
        }
    }
}

#[expect(clippy::suspicious_op_assign_impl)]
impl<T: Into<SignalSet>> ops::AddAssign<T> for SignalSet {
    fn add_assign(&mut self, rhs: T) {
        self.bits |= rhs.into().bits;
    }
}

impl<T: Into<SignalSet>> ops::Sub<T> for SignalSet {
    type Output = Self;

    fn sub(self, rhs: T) -> Self {
        SignalSet {
            bits: self.bits & !rhs.into().bits,
        }
    }
}

impl<T: Into<SignalSet>> ops::SubAssign<T> for SignalSet {
    fn sub_assign(&mut self, rhs: T) {
        self.bits &= !rhs.into().bits;
    }
}

impl ops::Not for SignalSet {
    type Output = Self;

    fn not(self) -> Self {
        SignalSet { bits: !self.bits }
    }
}

impl SignalSet {
    pub fn new_empty() -> Self {
        SignalSet { bits: 0 }
    }

    pub fn new_full() -> Self {
        SignalSet { bits: !0 }
    }

    pub const fn is_empty(&self) -> bool {
        self.bits == 0
    }

    pub const fn is_full(&self) -> bool {
        self.bits == !0
    }

    pub fn count(&self) -> usize {
        self.bits.count_ones() as usize
    }

    pub fn contains(&self, other: impl Into<Self>) -> bool {
        let other = other.into();
        self.bits & other.bits == other.bits
    }

    pub fn intersects(&self, other: impl Into<Self>) -> bool {
        let other = other.into();
        self.bits & other.bits != 0
    }
}

// This is to allow hexadecimally formatting a `SignalSet` when debug printing it.
impl LowerHex for SignalSet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        LowerHex::fmt(&self.bits, f) // delegate to u64's implementation
    }
}
