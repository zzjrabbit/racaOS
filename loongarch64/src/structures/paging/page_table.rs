use core::{
    fmt,
    ops::{Index, IndexMut},
};

use bitflags::bitflags;

use crate::{
    PhysAddr, PrivilegeLevel,
    structures::paging::{PageProperty, PhysFrame},
};

/// The error returned by the `PageTableEntry::frame` method.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameError {
    /// The entry does not have the `PRESENT` flag set, so it isn't currently mapped to a frame.
    FrameNotPresent,
    /// The entry does have the `HUGE_PAGE` flag set. The `frame` method has a standard 4KiB frame
    /// as return type, so a huge frame can't be returned.
    HugeFrame,
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageTableEntry {
    entry: u64,
}

impl PageTableEntry {
    /// Creates an unused page table entry.
    #[inline]
    pub const fn new() -> Self {
        PageTableEntry { entry: 0 }
    }

    /// Returns whether this entry is zero.
    #[inline]
    pub const fn is_unused(&self) -> bool {
        self.entry == 0
    }

    /// Sets this entry to zero.
    #[inline]
    pub fn set_unused(&mut self) {
        self.entry = 0;
    }

    #[inline]
    pub const fn frame(&self) -> PhysFrame {
        PhysFrame::containing_address(self.addr())
    }

    #[inline]
    pub const fn set_frame(&mut self, frame: PhysFrame, flags: PageTableFlags) {
        self.set_addr(frame.start_address(), flags);
    }

    /// Returns the physical address mapped by this entry, might be zero.
    #[inline]
    pub const fn addr(&self) -> PhysAddr {
        PhysAddr::new(self.entry & Self::physical_address_mask())
    }

    /// Map the entry to the specified physical address with the specified flags.
    #[inline]
    pub const fn set_addr(&mut self, addr: PhysAddr, flags: PageTableFlags) {
        self.entry = addr.as_u64() | flags.bits();
    }

    #[inline]
    pub const fn flags(&self) -> PageTableFlags {
        PageTableFlags::from_bits_truncate(self.entry & Self::flags_mask())
    }

    /// Sets the flags of this entry.
    #[inline]
    pub fn set_flags(&mut self, flags: PageTableFlags) {
        self.entry = u64::from(self.addr()) | flags.bits();
    }

    #[inline]
    pub const fn cache_policy(&self) -> CachePolicy {
        match (self.entry >> 4) & 0b11 {
            0 => CachePolicy::CoherentCached,
            1 => CachePolicy::StronglyOrderedUnCached,
            2 => CachePolicy::WeaklyOrderedUnCached,
            _ => CachePolicy::Reserved,
        }
    }

    #[inline]
    pub const fn set_cache_policy(&mut self, policy: CachePolicy) {
        self.entry = self.entry & !(0b11 << 4) | (policy as u64) << 4;
    }

    #[inline]
    pub const fn privilege(&self) -> PrivilegeLevel {
        match (self.entry >> 2) & 0b11 {
            0 => PrivilegeLevel::Privilege0,
            1 => PrivilegeLevel::Privilege1,
            2 => PrivilegeLevel::Privilege2,
            3 => PrivilegeLevel::Privilege3,
            _ => panic!("Unknown privilege level!"),
        }
    }

    #[inline]
    pub const fn set_privilege(&mut self, privilege: PrivilegeLevel) {
        self.entry = self.entry & !(0b11 << 2) | (privilege as u64) << 2;
    }

    /// If this is true, only the privilege level set can access this entry.
    /// If not, any higher privilege level can access this entry.
    #[inline]
    pub const fn privilege_restricted(&self) -> bool {
        (self.entry >> 63) & 0b1 == 1
    }

    #[inline]
    pub const fn set_privilege_restriction(&mut self, restricted: bool) {
        self.entry = self.entry & !(0b1 << 63) | (restricted as u64) << 63;
    }

    #[inline]
    pub fn property(&self) -> PageProperty {
        PageProperty::new()
            .add_flags(self.flags())
            .set_privilege(self.privilege())
            .set_privilege_restriction(self.privilege_restricted())
    }

    #[inline]
    pub fn set_property(&mut self, property: PageProperty) {
        let flags = property.flags();
        let privilege = property.privilege();
        let privilege_restriction = property.privilege_restriction();

        self.set_flags(flags);
        self.set_privilege(privilege);
        self.set_privilege_restriction(privilege_restriction);
    }

    #[inline(always)]
    const fn physical_address_mask() -> u64 {
        0x000f_ffff_ffff_f000u64
    }

    #[inline(always)]
    const fn flags_mask() -> u64 {
        0x0000_0000_0000_00ffu64
    }
}

impl Default for PageTableEntry {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("PageTableEntry");
        f.field("addr", &self.addr());
        f.field("flags", &self.flags());
        f.field("cache_policy", &self.cache_policy());
        f.field("privilege_restricted", &self.privilege_restricted());
        f.field("privilege", &self.privilege());
        f.finish()
    }
}

#[repr(u64)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CachePolicy {
    CoherentCached = 0,
    StronglyOrderedUnCached = 1,
    WeaklyOrderedUnCached = 2,
    Reserved = 3,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PageTableFlags: u64 {
        const VALID = 1 << 0;
        const DIRTY = 1 << 1;
        const HUGE_PAGE = 1 << 6;
        const PRESENT = 1 << 7;
        /// This flag is useless, set DIRTY to make it writable.
        const WRITABLE = 1 << 8;

        const GLOBAL = 1 << 6;
        const GLOBAL_FOR_HUGE_PAGE = 1 << 12;

        const NO_READ = 1 << 61;
        const NO_EXECUTE = 1 << 62;
    }
}

const ENTRY_COUNT: usize = 512;

#[repr(align(4096))]
#[repr(C)]
#[derive(Clone)]
pub struct PageTable {
    entries: [PageTableEntry; ENTRY_COUNT],
}

impl PageTable {
    /// Creates an empty page table.
    #[inline]
    pub const fn new() -> Self {
        const EMPTY: PageTableEntry = PageTableEntry::new();
        PageTable {
            entries: [EMPTY; ENTRY_COUNT],
        }
    }

    /// Clears all entries.
    #[inline]
    pub fn zero(&mut self) {
        for entry in self.iter_mut() {
            entry.set_unused();
        }
    }

    /// Returns an iterator over the entries of the page table.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &PageTableEntry> {
        (0..512).map(move |i| &self.entries[i])
    }

    /// Returns an iterator that allows modifying the entries of the page table.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PageTableEntry> {
        // Note that we intentionally don't just return `self.entries.iter()`:
        // Some users may choose to create a reference to a page table at
        // `0xffff_ffff_ffff_f000`. This causes problems because calculating
        // the end pointer of the page tables causes an overflow. Therefore
        // creating page tables at that address is unsound and must be avoided.
        // Unfortunately creating such page tables is quite common when
        // recursive page tables are used, so we try to avoid calculating the
        // end pointer if possible. `core::slice::Iter` calculates the end
        // pointer to determine when it should stop yielding elements. Because
        // we want to avoid calculating the end pointer, we don't use
        // `core::slice::Iter`, we implement our own iterator that doesn't
        // calculate the end pointer. This doesn't make creating page tables at
        // that address sound, but it avoids some easy to trigger
        // miscompilations.
        let ptr = self.entries.as_mut_ptr();
        (0..512).map(move |i| unsafe { &mut *ptr.add(i) })
    }

    /// Checks if the page table is empty (all entries are zero).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.iter().all(|entry| entry.is_unused())
    }
}

impl Index<usize> for PageTable {
    type Output = PageTableEntry;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<usize> for PageTable {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl Index<PageTableIndex> for PageTable {
    type Output = PageTableEntry;

    #[inline]
    fn index(&self, index: PageTableIndex) -> &Self::Output {
        &self.entries[usize::from(index)]
    }
}

impl IndexMut<PageTableIndex> for PageTable {
    #[inline]
    fn index_mut(&mut self, index: PageTableIndex) -> &mut Self::Output {
        &mut self.entries[usize::from(index)]
    }
}

impl Default for PageTable {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for PageTable {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.entries[..].fmt(f)
    }
}

/// A 9-bit index into a page table.
///
/// Can be used to select one of the 512 entries of a page table.
///
/// Guaranteed to only ever contain 0..512.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageTableIndex(u16);

impl PageTableIndex {
    /// Creates a new index from the given `u16`. Panics if the given value is >=512.
    #[inline]
    pub const fn new(index: u16) -> Self {
        assert!((index as usize) < ENTRY_COUNT);
        Self(index)
    }

    /// Creates a new index from the given `u16`. Throws away bits if the value is >=512.
    #[inline]
    pub const fn new_truncate(index: u16) -> Self {
        Self(index % ENTRY_COUNT as u16)
    }

    #[inline]
    pub(crate) const fn into_u64(self) -> u64 {
        self.0 as u64
    }
}

impl From<PageTableIndex> for u16 {
    #[inline]
    fn from(index: PageTableIndex) -> Self {
        index.0
    }
}

impl From<PageTableIndex> for u32 {
    #[inline]
    fn from(index: PageTableIndex) -> Self {
        u32::from(index.0)
    }
}

impl From<PageTableIndex> for u64 {
    #[inline]
    fn from(index: PageTableIndex) -> Self {
        index.into_u64()
    }
}

impl From<PageTableIndex> for usize {
    #[inline]
    fn from(index: PageTableIndex) -> Self {
        usize::from(index.0)
    }
}

/// A 12-bit offset into a 4KiB Page.
///
/// This type is returned by the `VirtAddr::page_offset` method.
///
/// Guaranteed to only ever contain 0..4096.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PageOffset(u16);

impl PageOffset {
    /// Creates a new offset from the given `u16`. Panics if the passed value is >=4096.
    #[inline]
    pub fn new(offset: u16) -> Self {
        assert!(offset < (1 << 12));
        Self(offset)
    }

    /// Creates a new offset from the given `u16`. Throws away bits if the value is >=4096.
    #[inline]
    pub const fn new_truncate(offset: u16) -> Self {
        Self(offset % (1 << 12))
    }
}

impl From<PageOffset> for u16 {
    #[inline]
    fn from(offset: PageOffset) -> Self {
        offset.0
    }
}

impl From<PageOffset> for u32 {
    #[inline]
    fn from(offset: PageOffset) -> Self {
        u32::from(offset.0)
    }
}

impl From<PageOffset> for u64 {
    #[inline]
    fn from(offset: PageOffset) -> Self {
        u64::from(offset.0)
    }
}

impl From<PageOffset> for usize {
    #[inline]
    fn from(offset: PageOffset) -> Self {
        usize::from(offset.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A value between 1 and 4.
pub enum PageTableLevel {
    /// Represents the level for a page table.
    One = 1,
    /// Represents the level for a page directory.
    Two,
    /// Represents the level for a page-directory pointer.
    Three,
    /// Represents the level for a page-map level-4.
    Four,
}

impl PageTableLevel {
    /// Returns the next lower level or `None` for level 1
    pub const fn next_lower_level(self) -> Option<Self> {
        match self {
            PageTableLevel::Four => Some(PageTableLevel::Three),
            PageTableLevel::Three => Some(PageTableLevel::Two),
            PageTableLevel::Two => Some(PageTableLevel::One),
            PageTableLevel::One => None,
        }
    }

    /// Returns the next higher level or `None` for level 4
    pub const fn next_higher_level(self) -> Option<Self> {
        match self {
            PageTableLevel::Four => None,
            PageTableLevel::Three => Some(PageTableLevel::Four),
            PageTableLevel::Two => Some(PageTableLevel::Three),
            PageTableLevel::One => Some(PageTableLevel::Two),
        }
    }

    /// Returns the alignment for the address space described by a table of this level.
    pub const fn table_address_space_alignment(self) -> u64 {
        1u64 << (self as u8 * 9 + 12)
    }

    /// Returns the alignment for the address space described by an entry in a table of this level.
    pub const fn entry_address_space_alignment(self) -> u64 {
        1u64 << (((self as u8 - 1) * 9) + 12)
    }
}
