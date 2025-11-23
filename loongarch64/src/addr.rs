use core::{
    fmt,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use bit_field::BitField;

use crate::structures::paging::{PageOffset, PageTableIndex, PageTableLevel};

const ADDRESS_SPACE_SIZE: u64 = 0x1_0000_0000_0000;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PhysAddr(u64);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VirtAddr(u64);

impl VirtAddr {
    pub const fn new(addr: u64) -> Self {
        VirtAddr(addr)
    }

    pub const fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    pub const fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }
}

impl PhysAddr {
    pub const fn new(addr: u64) -> Self {
        PhysAddr(addr)
    }
}

macro_rules! impl_from_num {
    ($addr: ident, $num: ident) => {
        impl From<$num> for $addr {
            fn from(addr: $num) -> Self {
                $addr(addr as u64)
            }
        }

        impl From<$addr> for $num {
            fn from(addr: $addr) -> Self {
                addr.0 as $num
            }
        }
    };
}

impl_from_num!(PhysAddr, u64);
impl_from_num!(VirtAddr, u64);

impl_from_num!(PhysAddr, usize);
impl_from_num!(VirtAddr, usize);

impl_from_num!(PhysAddr, u32);
impl_from_num!(VirtAddr, u32);

impl_from_num!(PhysAddr, u16);
impl_from_num!(VirtAddr, u16);

impl_from_num!(PhysAddr, u8);
impl_from_num!(VirtAddr, u8);

impl_from_num!(PhysAddr, i64);
impl_from_num!(VirtAddr, i64);

impl_from_num!(PhysAddr, i32);
impl_from_num!(VirtAddr, i32);

impl_from_num!(PhysAddr, i16);
impl_from_num!(VirtAddr, i16);

impl_from_num!(PhysAddr, i8);
impl_from_num!(VirtAddr, i8);

impl fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("VirtAddr")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

impl fmt::Binary for VirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Binary::fmt(&self.0, f)
    }
}

impl fmt::LowerHex for VirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::Octal for VirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Octal::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for VirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

impl fmt::Pointer for VirtAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&(self.0 as *const ()), f)
    }
}

impl Add<u64> for VirtAddr {
    type Output = Self;

    /// Add an offset to a virtual address.
    ///
    /// This function performs normal arithmetic addition and doesn't jump the
    /// address gap. If you're looking for a successor operation that jumps the
    /// address gap, use [`Step::forward`].
    ///
    /// # Panics
    ///
    /// This function will panic on overflow or if the result is not a
    /// canonical address.
    #[inline]
    fn add(self, rhs: u64) -> Self::Output {
        VirtAddr::new(
            self.0
                .checked_add(rhs)
                .expect("attempt to add with overflow"),
        )
    }
}

impl AddAssign<u64> for VirtAddr {
    /// Add an offset to a virtual address.
    ///
    /// This function performs normal arithmetic addition and doesn't jump the
    /// address gap. If you're looking for a successor operation that jumps the
    /// address gap, use [`Step::forward`].
    ///
    /// # Panics
    ///
    /// This function will panic on overflow or if the result is not a
    /// canonical address.
    #[inline]
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl Sub<u64> for VirtAddr {
    type Output = Self;

    /// Subtract an offset from a virtual address.
    ///
    /// This function performs normal arithmetic subtraction and doesn't jump
    /// the address gap. If you're looking for a predecessor operation that
    /// jumps the address gap, use [`Step::backward`].
    ///
    /// # Panics
    ///
    /// This function will panic on overflow or if the result is not a
    /// canonical address.
    #[inline]
    fn sub(self, rhs: u64) -> Self::Output {
        VirtAddr::new(
            self.0
                .checked_sub(rhs)
                .expect("attempt to subtract with overflow"),
        )
    }
}

impl SubAssign<u64> for VirtAddr {
    /// Subtract an offset from a virtual address.
    ///
    /// This function performs normal arithmetic subtraction and doesn't jump
    /// the address gap. If you're looking for a predecessor operation that
    /// jumps the address gap, use [`Step::backward`].
    ///
    /// # Panics
    ///
    /// This function will panic on overflow or if the result is not a
    /// canonical address.
    #[inline]
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}

impl Sub<VirtAddr> for VirtAddr {
    type Output = u64;

    /// Returns the difference between two addresses.
    ///
    /// # Panics
    ///
    /// This function will panic on overflow.
    #[inline]
    fn sub(self, rhs: VirtAddr) -> Self::Output {
        u64::from(self)
            .checked_sub(rhs.into())
            .expect("attempt to subtract with overflow")
    }
}

impl fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("PhysAddr")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

impl fmt::Binary for PhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Binary::fmt(&self.0, f)
    }
}

impl fmt::LowerHex for PhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::Octal for PhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Octal::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for PhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

impl fmt::Pointer for PhysAddr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&(self.0 as *const ()), f)
    }
}

impl Add<u64> for PhysAddr {
    type Output = Self;
    #[inline]
    fn add(self, rhs: u64) -> Self::Output {
        PhysAddr::new(self.0.checked_add(rhs).unwrap())
    }
}

impl AddAssign<u64> for PhysAddr {
    #[inline]
    fn add_assign(&mut self, rhs: u64) {
        *self = *self + rhs;
    }
}

impl Sub<u64> for PhysAddr {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: u64) -> Self::Output {
        PhysAddr::new(self.0.checked_sub(rhs).unwrap())
    }
}

impl SubAssign<u64> for PhysAddr {
    #[inline]
    fn sub_assign(&mut self, rhs: u64) {
        *self = *self - rhs;
    }
}

impl Sub<PhysAddr> for PhysAddr {
    type Output = u64;
    #[inline]
    fn sub(self, rhs: PhysAddr) -> Self::Output {
        u64::from(self).checked_sub(rhs.into()).unwrap()
    }
}

impl PhysAddr {
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// Aligns the virtual address upwards to the given alignment.
    ///
    /// See the `align_up` function for more information.
    ///
    /// # Panics
    ///
    /// This function panics if the resulting address is higher than
    /// `0xffff_ffff_ffff_ffff`.
    #[inline]
    pub fn align_up<U>(self, align: U) -> Self
    where
        U: Into<u64>,
    {
        Self::new(align_up(self.0, align.into()))
    }

    /// Aligns the virtual address downwards to the given alignment.
    ///
    /// See the `align_down` function for more information.
    #[inline]
    pub fn align_down<U>(self, align: U) -> Self
    where
        U: Into<u64>,
    {
        self.align_down_u64(align.into())
    }

    /// Aligns the virtual address downwards to the given alignment.
    ///
    /// See the `align_down` function for more information.
    #[inline]
    pub(crate) const fn align_down_u64(self, align: u64) -> Self {
        Self::new(align_down(self.0, align))
    }

    /// Checks whether the virtual address has the demanded alignment.
    #[inline]
    pub fn is_aligned<U>(self, align: U) -> bool
    where
        U: Into<u64>,
    {
        self.is_aligned_u64(align.into())
    }

    /// Checks whether the virtual address has the demanded alignment.
    #[inline]
    pub(crate) const fn is_aligned_u64(self, align: u64) -> bool {
        self.align_down_u64(align).as_u64() == self.as_u64()
    }
}

impl VirtAddr {
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    /// Aligns the virtual address upwards to the given alignment.
    ///
    /// See the `align_up` function for more information.
    ///
    /// # Panics
    ///
    /// This function panics if the resulting address is higher than
    /// `0xffff_ffff_ffff_ffff`.
    #[inline]
    pub fn align_up<U>(self, align: U) -> Self
    where
        U: Into<u64>,
    {
        VirtAddr::new(align_up(self.0, align.into()))
    }

    /// Aligns the virtual address downwards to the given alignment.
    ///
    /// See the `align_down` function for more information.
    #[inline]
    pub fn align_down<U>(self, align: U) -> Self
    where
        U: Into<u64>,
    {
        self.align_down_u64(align.into())
    }

    /// Aligns the virtual address downwards to the given alignment.
    ///
    /// See the `align_down` function for more information.
    #[inline]
    pub(crate) const fn align_down_u64(self, align: u64) -> Self {
        VirtAddr::new(align_down(self.0, align))
    }

    /// Checks whether the virtual address has the demanded alignment.
    #[inline]
    pub fn is_aligned<U>(self, align: U) -> bool
    where
        U: Into<u64>,
    {
        self.is_aligned_u64(align.into())
    }

    /// Checks whether the virtual address has the demanded alignment.
    #[inline]
    pub(crate) const fn is_aligned_u64(self, align: u64) -> bool {
        self.align_down_u64(align).as_u64() == self.as_u64()
    }

    /// Returns the 12-bit page offset of this virtual address.
    #[inline]
    pub const fn page_offset(self) -> PageOffset {
        PageOffset::new_truncate(self.0 as u16)
    }

    /// Returns the 9-bit level 1 page table index.
    #[inline]
    pub const fn p1_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12) as u16)
    }

    /// Returns the 9-bit level 2 page table index.
    #[inline]
    pub const fn p2_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12 >> 9) as u16)
    }

    /// Returns the 9-bit level 3 page table index.
    #[inline]
    pub const fn p3_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12 >> 9 >> 9) as u16)
    }

    /// Returns the 9-bit level 4 page table index.
    #[inline]
    pub const fn p4_index(self) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12 >> 9 >> 9 >> 9) as u16)
    }

    /// Returns the 9-bit level page table index.
    #[inline]
    pub const fn page_table_index(self, level: PageTableLevel) -> PageTableIndex {
        PageTableIndex::new_truncate((self.0 >> 12 >> ((level as u8 - 1) * 9)) as u16)
    }

    #[inline]
    pub(crate) fn forward_checked_impl(start: Self, count: usize) -> Option<Self> {
        Self::forward_checked_u64(start, u64::try_from(count).ok()?)
    }

    /// An implementation of forward_checked that takes u64 instead of usize.
    #[inline]
    pub(crate) fn forward_checked_u64(start: Self, count: u64) -> Option<Self> {
        if count > ADDRESS_SPACE_SIZE {
            return None;
        }

        let mut addr = start.0.checked_add(count)?;

        match addr.get_bits(47..) {
            0x1 => {
                // Jump the gap by sign extending the 47th bit.
                addr.set_bits(47.., 0x1ffff);
            }
            0x2 => {
                // Address overflow
                return None;
            }
            _ => {}
        }

        Some(Self::new(addr))
    }
}

/// Align address downwards.
///
/// Returns the greatest `x` with alignment `align` so that `x <= addr`.
///
/// Panics if the alignment is not a power of two.
#[inline]
pub const fn align_down(addr: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two(), "`align` must be a power of two");
    addr & !(align - 1)
}

/// Align address upwards.
///
/// Returns the smallest `x` with alignment `align` so that `x >= addr`.
///
/// Panics if the alignment is not a power of two or if an overflow occurs.
#[inline]
pub const fn align_up(addr: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two(), "`align` must be a power of two");
    let align_mask = align - 1;
    if addr & align_mask == 0 {
        addr // already aligned
    } else {
        // FIXME: Replace with .expect, once `Option::expect` is const.
        if let Some(aligned) = (addr | align_mask).checked_add(1) {
            aligned
        } else {
            panic!("attempt to add with overflow")
        }
    }
}
